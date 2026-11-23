//! Peer serialization/deserialization.
//!
//! Source: `storage/serialize_peer.{h,cpp}`.

use crate::image_location::ImageLocation;
use models_peers::{
    deserialize_peer_id, serialize_peer_id, ChannelDataFlags, ChatDataFlags, PeerData, PeerId,
    UserDataFlags, UserId,
};
use storage_serialize::{ByteArrayReader, ByteArrayWriter};

pub const VERSION_TAG: u64 = 0x77FF_FFFF_FFFF_FFFF;
pub const VERSION: i32 = 3;

pub fn write_peer(writer: &mut ByteArrayWriter, peer: &PeerData) {
    writer.write_u64(serialize_peer_id(peer.id));
    writer.write_u64(VERSION_TAG);
    writer.write_i32(VERSION);
    writer.write_u64(peer.userpic_photo_id);

    // Dummy userpic location
    let dummy_loc = ImageLocation::default();
    dummy_loc.write(writer);
    writer.write_i32(if peer.userpic_has_video { 1 } else { 0 });

    if let Some(user) = peer.as_user() {
        let bot_inline_placeholder = user
            .bot_info
            .as_ref()
            .map(|b| b.inline_placeholder.as_str())
            .unwrap_or("");
        writer.write_string(Some(&user.first_name));
        writer.write_string(Some(&user.last_name));
        writer.write_string(Some(&user.phone));
        writer.write_string(Some(&user.username));
        writer.write_u64(user.access_hash);
        writer.write_i32(user.flags.0 as i32);
        writer.write_string(Some(bot_inline_placeholder));
        writer.write_u32(user.lastseen);
        writer.write_i32(if user.is_contact { 1 } else { 0 });
        writer.write_i32(user.bot_info.as_ref().map(|b| b.version).unwrap_or(-1));
        writer.write_i32(
            if user
                .bot_info
                .as_ref()
                .map(|b| b.supports_guest_chat)
                .unwrap_or(false)
            {
                1
            } else {
                0
            },
        );
    } else if let Some(chat) = peer.as_chat() {
        let field1 = (chat.creator.bare & 0xFFFF_FFFF) as i32;
        let field2 = ((chat.creator.bare >> 32) << 8) as i32;
        writer.write_string(Some(&chat.name));
        writer.write_i32(chat.count);
        writer.write_i32(chat.date);
        writer.write_i32(chat.version);
        writer.write_i32(field1);
        writer.write_i32(field2);
        writer.write_u32(chat.flags.0);
        writer.write_string(Some(&chat.invite_link));
    } else if let Some(channel) = peer.as_channel() {
        writer.write_string(Some(&channel.name));
        writer.write_u64(channel.access_hash);
        writer.write_i32(channel.date);
        writer.write_i32(0); // legacy version
        writer.write_i32(0); // legacy
        writer.write_u32((channel.flags.0 & 0xFFFF_FFFF) as u32);
        writer.write_string(Some(&channel.invite_link));
    }
}

pub fn read_peer(reader: &mut ByteArrayReader, stream_version: i32) -> Option<PeerData> {
    let peer_id_serialized = reader.read_u64()?;
    let version_tag = reader.read_u64()?;
    let peer_id = deserialize_peer_id(peer_id_serialized);
    if peer_id == PeerId::ZERO {
        return None;
    }

    let mut version = 0;
    let mut photo_id = 0;
    if version_tag == VERSION_TAG {
        version = reader.read_i32()?;
        photo_id = reader.read_u64()?;
    } else {
        photo_id = version_tag;
    }

    let _userpic = ImageLocation::read(reader, stream_version)?;
    let mut photo_has_video = 0;
    if version > 0 {
        photo_has_video = reader.read_i32()?;
    }

    let mut result = PeerData::new(peer_id);
    result.userpic_photo_id = photo_id;
    result.userpic_has_video = photo_has_video == 1;

    if peer_id.is_user() {
        let first = reader.read_string()?;
        let last = reader.read_string()?;
        let phone = reader.read_string()?;
        let username = reader.read_string()?;
        let access = reader.read_u64()?;
        let mut flags = 0;
        if stream_version >= 9012 {
            flags = reader.read_i32()?;
        }
        let mut inline_placeholder = String::new();
        if stream_version >= 9016 {
            inline_placeholder = reader.read_string()?;
        }
        let lastseen = reader.read_u32()?;
        let contact = reader.read_i32()?;
        let bot_info_version = reader.read_i32()?;
        let mut supports_guest_chat = 0;
        if version > 2 {
            supports_guest_chat = reader.read_i32()?;
        }

        if let Some(user) = result.as_user_mut() {
            user.first_name = first;
            user.last_name = last;
            user.phone = phone;
            user.username = username;
            user.access_hash = access;
            user.flags = UserDataFlags(flags as u32);
            user.lastseen = lastseen;
            user.is_contact = contact == 1;
            if bot_info_version >= 0 {
                user.bot_info = Some(models_peers::BotInfo {
                    version: bot_info_version,
                    inline_placeholder,
                    supports_guest_chat: supports_guest_chat == 1,
                    description: String::new(),
                });
            }
        }
    } else if peer_id.is_chat() {
        let name = reader.read_string()?;
        let count = reader.read_i32()?;
        let date = reader.read_i32()?;
        let version = reader.read_i32()?;
        let field1 = reader.read_i32()?;
        let field2 = reader.read_i32()?;
        let flags = reader.read_u32()?;
        let invite_link = reader.read_string()?;

        let creator_bare = (field1 as u32 as u64) | (((field2 as u32 >> 8) as u64) << 32);

        if let Some(chat) = result.as_chat_mut() {
            chat.name = name;
            chat.count = count;
            chat.date = date;
            chat.version = version;
            chat.creator = UserId::new(creator_bare);
            chat.flags = ChatDataFlags(flags);
            chat.invite_link = invite_link;
        }
    } else if peer_id.is_channel() {
        let name = reader.read_string()?;
        let access = reader.read_u64()?;
        let date = reader.read_i32()?;
        let _v1 = reader.read_i32()?;
        let _v2 = reader.read_i32()?;
        let flags = reader.read_u32()?;
        let invite_link = reader.read_string()?;

        if let Some(channel) = result.as_channel_mut() {
            channel.name = name;
            channel.access_hash = access;
            channel.date = date;
            channel.flags = ChannelDataFlags(flags as u64);
            channel.invite_link = invite_link;
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_serialization_roundtrip() {
        let mut peer = PeerData::new(PeerId::from_user(UserId::new(54321)));
        if let Some(user) = peer.as_user_mut() {
            user.first_name = "John".to_string();
            user.last_name = "Doe".to_string();
            user.phone = "+123456789".to_string();
            user.username = "johndoe".to_string();
            user.access_hash = 0xDEADBEEF;
            user.is_contact = true;
        }

        let mut writer = ByteArrayWriter::new();
        write_peer(&mut writer, &peer);

        let mut reader = ByteArrayReader::from_slice(writer.result());
        let read = read_peer(&mut reader, 1000000).unwrap();

        assert_eq!(read.id, peer.id);
        let u = read.as_user().unwrap();
        assert_eq!(u.first_name, "John");
        assert_eq!(u.last_name, "Doe");
        assert_eq!(u.phone, "+123456789");
        assert_eq!(u.username, "johndoe");
        assert_eq!(u.access_hash, 0xDEADBEEF);
        assert!(u.is_contact);
    }
}
