//! Data::AutoDownload models and serialization matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/data/data_auto_download.{h,cpp}`).

use std::collections::BTreeMap;
use models_peers::id::{deserialize_peer_id, serialize_peer_id, PeerId};
use storage_serialize::{ByteArrayReader, ByteArrayWriter};

pub const MAX_BYTES_LIMIT: i64 = 8000 * (512 * 1024);
pub const DEFAULT_MAX_SIZE: i64 = 8 * (1024 * 1024);
pub const DEFAULT_AUTOPLAY_SIZE: i64 = 50 * (1024 * 1024);

pub const VERSION_1: i8 = 1;
pub const VERSION_2: i8 = 2;
pub const VERSION_3: i8 = 3;
pub const CURRENT_VERSION: i8 = VERSION_3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Source {
    User = 0x00,
    Group = 0x01,
    Channel = 0x02,
}

pub const SOURCES_COUNT: usize = 3;
pub const ALL_SOURCES: [Source; SOURCES_COUNT] = [Source::User, Source::Group, Source::Channel];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Type {
    Photo = 0x00,
    AutoPlayVideo = 0x01,
    VoiceMessage = 0x02,
    AutoPlayVideoMessage = 0x03,
    Music = 0x04,
    AutoPlayGIF = 0x05,
    File = 0x06,
}

pub const TYPES_COUNT: usize = 7;
pub const ALL_TYPES: [Type; TYPES_COUNT] = [
    Type::Photo,
    Type::AutoPlayVideo,
    Type::VoiceMessage,
    Type::AutoPlayVideoMessage,
    Type::Music,
    Type::AutoPlayGIF,
    Type::File,
];

pub const AUTOPLAY_TYPES: [Type; 3] = [
    Type::AutoPlayVideo,
    Type::AutoPlayVideoMessage,
    Type::AutoPlayGIF,
];

pub const STREAMED_TYPES: [Type; 2] = [
    Type::VoiceMessage,
    Type::Music,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i8)]
pub enum Override {
    #[default]
    Default = 0,
    ForceAllow = 1,
    ForceDeny = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Single {
    limit: i32,
}

impl Default for Single {
    fn default() -> Self {
        Self { limit: -1 }
    }
}

impl Single {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bytes_limit(&mut self, bytes_limit: i64) {
        assert!(bytes_limit >= 0 && bytes_limit <= MAX_BYTES_LIMIT);
        self.limit = bytes_limit as u32 as i32;
    }

    pub fn has_value(&self) -> bool {
        self.limit != -1
    }

    pub fn should_download(&self, file_size: i64) -> bool {
        assert!(self.has_value());
        let real_limit = self.bytes_limit();
        real_limit > 0 && file_size <= real_limit
    }

    pub fn bytes_limit(&self) -> i64 {
        assert!(self.has_value());
        (self.limit as u32) as i64
    }

    pub fn serialize(&self) -> i32 {
        self.limit
    }

    pub fn set_from_serialized(&mut self, serialized: i32) -> bool {
        let real_limit = serialized as u32 as i64;
        if serialized != -1 && real_limit > MAX_BYTES_LIMIT {
            return false;
        }
        self.limit = serialized;
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    data: [Single; TYPES_COUNT],
}

impl Default for Set {
    fn default() -> Self {
        Self {
            data: [Single::default(); TYPES_COUNT],
        }
    }
}

impl Set {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single(&self, type_: Type) -> &Single {
        &self.data[type_ as usize]
    }

    pub fn single_mut(&mut self, type_: Type) -> &mut Single {
        &mut self.data[type_ as usize]
    }

    pub fn set_bytes_limit(&mut self, type_: Type, bytes_limit: i64) {
        self.single_mut(type_).set_bytes_limit(bytes_limit);
    }

    pub fn has_value(&self, type_: Type) -> bool {
        self.single(type_).has_value()
    }

    pub fn should_download(&self, type_: Type, file_size: i64) -> bool {
        self.single(type_).should_download(file_size)
    }

    pub fn bytes_limit(&self, type_: Type) -> i64 {
        self.single(type_).bytes_limit()
    }

    pub fn serialize(&self, type_: Type) -> i32 {
        self.single(type_).serialize()
    }

    pub fn set_from_serialized(&mut self, type_: Type, serialized: i32) -> bool {
        self.single_mut(type_).set_from_serialized(serialized)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Full {
    data: [Set; SOURCES_COUNT],
    peer_overrides: BTreeMap<PeerId, Override>,
}

impl Default for Full {
    fn default() -> Self {
        Self {
            data: [Set::default(), Set::default(), Set::default()],
            peer_overrides: BTreeMap::new(),
        }
    }
}

pub fn defaults() -> Full {
    let mut result = Full::full_disabled();
    for source in ALL_SOURCES {
        set_defaults_for_source(&mut result, source);
    }
    result
}

pub fn set_defaults_for_source(data: &mut Full, source: Source) {
    data.set_bytes_limit(source, Type::Photo, DEFAULT_MAX_SIZE);
    data.set_bytes_limit(source, Type::VoiceMessage, DEFAULT_MAX_SIZE);
    data.set_bytes_limit(source, Type::AutoPlayVideoMessage, DEFAULT_AUTOPLAY_SIZE);
    data.set_bytes_limit(source, Type::AutoPlayGIF, DEFAULT_AUTOPLAY_SIZE);
    let channels_file_limit = if source == Source::Channel {
        0
    } else {
        DEFAULT_MAX_SIZE
    };
    data.set_bytes_limit(source, Type::File, channels_file_limit);
    data.set_bytes_limit(source, Type::AutoPlayVideo, DEFAULT_AUTOPLAY_SIZE);
    data.set_bytes_limit(source, Type::Music, channels_file_limit);
}

pub fn set_disabled_for_source(data: &mut Full, source: Source) {
    for type_ in ALL_TYPES {
        data.set_bytes_limit(source, type_, 0);
    }
}

pub fn has_enabled_types(data: &Full, source: Source) -> bool {
    ALL_TYPES.iter().any(|&type_| {
        !STREAMED_TYPES.contains(&type_) && data.bytes_limit(source, type_) > 0
    })
}

impl Full {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn full_disabled() -> Self {
        let mut result = Self::new();
        for source in ALL_SOURCES {
            for type_ in ALL_TYPES {
                result.set_bytes_limit(source, type_, 0);
            }
        }
        result
    }

    pub fn set(&self, source: Source) -> &Set {
        &self.data[source as usize]
    }

    pub fn set_mut(&mut self, source: Source) -> &mut Set {
        &mut self.data[source as usize]
    }

    pub fn set_or_default(&self, source: Source, type_: Type) -> &Set {
        let my = self.set(source);
        if my.has_value(type_) {
            my
        } else {
            &DEFAULTS.data[source as usize]
        }
    }

    pub fn set_bytes_limit(&mut self, source: Source, type_: Type, bytes_limit: i64) {
        self.set_mut(source).set_bytes_limit(type_, bytes_limit);
    }

    pub fn should_download(&self, source: Source, type_: Type, file_size: i64) -> bool {
        if STREAMED_TYPES.contains(&type_) {
            return false;
        }
        self.set_or_default(source, type_).should_download(type_, file_size)
    }

    pub fn bytes_limit(&self, source: Source, type_: Type) -> i64 {
        self.set_or_default(source, type_).bytes_limit(type_)
    }

    pub fn set_peer_override(&mut self, peer_id: PeerId, value: Override) {
        if value == Override::Default {
            self.peer_overrides.remove(&peer_id);
        } else {
            self.peer_overrides.insert(peer_id, value);
        }
    }

    pub fn peer_override(&self, peer_id: PeerId) -> Override {
        self.peer_overrides.get(&peer_id).copied().unwrap_or(Override::Default)
    }

    pub fn peer_overrides(&self) -> &BTreeMap<PeerId, Override> {
        &self.peer_overrides
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut writer = ByteArrayWriter::new();
        writer.write_u8(CURRENT_VERSION as u8);
        for source in ALL_SOURCES {
            for type_ in ALL_TYPES {
                writer.write_i32(self.set(source).serialize(type_));
            }
        }
        writer.write_i32(self.peer_overrides.len() as i32);
        for (&peer_id, &override_val) in &self.peer_overrides {
            writer.write_u64(serialize_peer_id(peer_id));
            writer.write_u8(override_val as u8);
        }
        writer.into_result()
    }

    pub fn set_from_serialized(&mut self, serialized: &[u8]) -> bool {
        if serialized.is_empty() {
            return false;
        }

        let mut reader = ByteArrayReader::from_slice(serialized);
        let version = match reader.read_u8() {
            Some(v) => v as i8,
            None => return false,
        };

        if version != VERSION_3 && version != VERSION_2 && version != VERSION_1 {
            return false;
        }

        let mut temp = Full::new();
        for source in ALL_SOURCES {
            for type_ in ALL_TYPES {
                let value = match reader.read_i32() {
                    Some(v) => v,
                    None => return false,
                };
                if !temp.set_mut(source).set_from_serialized(type_, value) {
                    return false;
                }
            }
        }

        if version == VERSION_1 {
            for source in ALL_SOURCES {
                for type_ in AUTOPLAY_TYPES {
                    let current = temp.bytes_limit(source, type_);
                    temp.set_bytes_limit(source, type_, std::cmp::max(current, DEFAULT_AUTOPLAY_SIZE));
                }
            }
        }

        if version >= VERSION_3 && !reader.at_end() {
            let count = match reader.read_i32() {
                Some(c) => c,
                None => return false,
            };
            if count < 0 {
                return false;
            }
            for _ in 0..count {
                let serialized_peer_id = match reader.read_u64() {
                    Some(p) => p,
                    None => return false,
                };
                let raw_override = match reader.read_u8() {
                    Some(o) => o as i8,
                    None => return false,
                };
                let value = match raw_override {
                    1 => Override::ForceAllow,
                    2 => Override::ForceDeny,
                    _ => Override::Default,
                };
                if value != Override::Default {
                    temp.set_peer_override(deserialize_peer_id(serialized_peer_id), value);
                }
            }
        }

        self.data = temp.data;
        self.peer_overrides = temp.peer_overrides;
        true
    }
}

// Global defaults instance
use std::sync::LazyLock;
static DEFAULTS: LazyLock<Full> = LazyLock::new(defaults);

pub fn force_allow_limit(data: &Full, source: Source, type_: Type) -> i64 {
    let user = data.bytes_limit(source, type_);
    if user > 0 {
        user
    } else {
        defaults().bytes_limit(source, type_)
    }
}

pub fn force_allowed(data: &Full, source: Source, type_: Type, file_size: i64) -> bool {
    if STREAMED_TYPES.contains(&type_) {
        return false;
    }
    let limit = force_allow_limit(data, source, type_);
    limit > 0 && file_size <= limit
}

pub fn with_disabled_auto_play(data: &Full) -> Full {
    let mut result = data.clone();
    for source in ALL_SOURCES {
        for type_ in AUTOPLAY_TYPES {
            result.set_bytes_limit(source, type_, 0);
        }
    }
    let to_clear: Vec<PeerId> = data
        .peer_overrides()
        .iter()
        .filter_map(|(&id, &val)| if val == Override::ForceAllow { Some(id) } else { None })
        .collect();
    for id in to_clear {
        result.set_peer_override(id, Override::Default);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::id::UserId;

    #[test]
    fn test_single_limits() {
        let mut single = Single::new();
        assert!(!single.has_value());

        single.set_bytes_limit(1024);
        assert!(single.has_value());
        assert_eq!(single.bytes_limit(), 1024);
        assert!(single.should_download(500));
        assert!(single.should_download(1024));
        assert!(!single.should_download(1025));

        let ser = single.serialize();
        let mut single2 = Single::new();
        assert!(single2.set_from_serialized(ser));
        assert_eq!(single2.bytes_limit(), 1024);
    }

    #[test]
    fn test_full_defaults() {
        let def = defaults();
        assert_eq!(def.bytes_limit(Source::User, Type::Photo), DEFAULT_MAX_SIZE);
        assert_eq!(def.bytes_limit(Source::Channel, Type::File), 0);
        assert_eq!(def.bytes_limit(Source::Group, Type::File), DEFAULT_MAX_SIZE);

        assert!(def.should_download(Source::User, Type::Photo, 1000));
        assert!(!def.should_download(Source::Channel, Type::File, 1000));
        assert!(!def.should_download(Source::User, Type::VoiceMessage, 1000)); // Streamed
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut full = defaults();
        let user_id = PeerId::from_user(UserId::new(12345));
        full.set_peer_override(user_id, Override::ForceAllow);
        full.set_bytes_limit(Source::Channel, Type::File, 5000);

        let bytes = full.serialize();
        let mut restored = Full::new();
        assert!(restored.set_from_serialized(&bytes));

        assert_eq!(restored.bytes_limit(Source::Channel, Type::File), 5000);
        assert_eq!(restored.peer_override(user_id), Override::ForceAllow);
    }

    #[test]
    fn test_with_disabled_auto_play() {
        let mut full = defaults();
        let user_id = PeerId::from_user(UserId::new(12345));
        full.set_peer_override(user_id, Override::ForceAllow);

        let disabled = with_disabled_auto_play(&full);
        assert_eq!(disabled.bytes_limit(Source::User, Type::AutoPlayVideo), 0);
        assert_eq!(disabled.bytes_limit(Source::User, Type::Photo), DEFAULT_MAX_SIZE);
        assert_eq!(disabled.peer_override(user_id), Override::Default);
    }
}
