//! MTProto server configuration (`help.getConfig`) and serialization.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/mtproto/mtproto_config.{h,cpp}`.

use mtproto_dc_options::{DcOptions, Environment};
use storage_serialize::{ByteArrayReader, ByteArrayWriter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFields {
    pub chat_size_max: i32,
    pub megagroup_size_max: i32,
    pub forwarded_count_max: i32,
    pub online_update_period_ms: i32,
    pub offline_blur_timeout_ms: i32,
    pub offline_idle_timeout_ms: i32,
    pub online_focus_timeout_ms: i32,
    pub online_cloud_timeout_ms: i32,
    pub notify_cloud_delay_ms: i32,
    pub notify_default_delay_ms: i32,
    pub edit_time_limit_secs: i32,
    pub revoke_time_limit_secs: i32,
    pub revoke_private_time_limit_secs: i32,
    pub revoke_private_inbox: bool,
    pub stickers_recent_limit: i32,
    pub internal_links_domain: String,
    pub channels_read_media_period_secs: i32,
    pub call_receive_timeout_ms: i32,
    pub call_ring_timeout_ms: i32,
    pub call_connect_timeout_ms: i32,
    pub call_packet_timeout_ms: i32,
    pub web_file_dc_id: i32,
    pub txt_domain_string: String,
    pub blocked_mode: bool,
    pub caption_length_max: i32,
    pub rating_decay_secs: i32,
    pub reaction_default_emoji: String,
    pub reaction_default_custom: u64,
    pub autologin_token: String,
    pub gif_search_username: String,
}

impl ConfigFields {
    pub fn new(env: Environment) -> Self {
        let web_file_dc_id = match env {
            Environment::Production => 4,
            Environment::Test => 2,
        };
        Self {
            chat_size_max: 200,
            megagroup_size_max: 10_000,
            forwarded_count_max: 100,
            online_update_period_ms: 120_000,
            offline_blur_timeout_ms: 5_000,
            offline_idle_timeout_ms: 30_000,
            online_focus_timeout_ms: 1_000,
            online_cloud_timeout_ms: 300_000,
            notify_cloud_delay_ms: 30_000,
            notify_default_delay_ms: 1_500,
            edit_time_limit_secs: 172_800,
            revoke_time_limit_secs: 172_800,
            revoke_private_time_limit_secs: 172_800,
            revoke_private_inbox: false,
            stickers_recent_limit: 30,
            internal_links_domain: "https://t.me/".to_string(),
            channels_read_media_period_secs: 86_400 * 7,
            call_receive_timeout_ms: 20_000,
            call_ring_timeout_ms: 90_000,
            call_connect_timeout_ms: 30_000,
            call_packet_timeout_ms: 10_000,
            web_file_dc_id,
            txt_domain_string: String::new(),
            blocked_mode: false,
            caption_length_max: 1024,
            rating_decay_secs: 2_419_200,
            reaction_default_emoji: String::new(),
            reaction_default_custom: 0,
            autologin_token: String::new(),
            gif_search_username: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MtpConfig {
    pub dc_options: DcOptions,
    pub fields: ConfigFields,
}

impl MtpConfig {
    pub fn new(env: Environment) -> Self {
        Self {
            dc_options: DcOptions::new(env),
            fields: ConfigFields::new(env),
        }
    }

    pub fn environment(&self) -> Environment {
        self.dc_options.environment()
    }

    pub fn is_test_mode(&self) -> bool {
        self.dc_options.is_test_mode()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut w = ByteArrayWriter::new();
        w.write_i32(self.fields.chat_size_max);
        w.write_i32(self.fields.megagroup_size_max);
        w.write_i32(self.fields.forwarded_count_max);
        w.write_i32(self.fields.edit_time_limit_secs);
        w.write_i32(self.fields.revoke_time_limit_secs);
        w.write_i32(self.fields.stickers_recent_limit);
        w.write_i32(self.fields.caption_length_max);
        w.write_i32(self.fields.web_file_dc_id);
        w.write_string(Some(&self.fields.internal_links_domain));
        w.write_string(Some(&self.fields.txt_domain_string));
        w.write_string(Some(&self.fields.reaction_default_emoji));
        w.write_u64(self.fields.reaction_default_custom);
        w.into_result()
    }

    pub fn from_serialized(env: Environment, bytes: &[u8]) -> Option<Self> {
        let mut r = ByteArrayReader::from_slice(bytes);
        let mut cfg = Self::new(env);
        cfg.fields.chat_size_max = r.read_i32()?;
        cfg.fields.megagroup_size_max = r.read_i32()?;
        cfg.fields.forwarded_count_max = r.read_i32()?;
        cfg.fields.edit_time_limit_secs = r.read_i32()?;
        cfg.fields.revoke_time_limit_secs = r.read_i32()?;
        cfg.fields.stickers_recent_limit = r.read_i32()?;
        cfg.fields.caption_length_max = r.read_i32()?;
        cfg.fields.web_file_dc_id = r.read_i32()?;
        cfg.fields.internal_links_domain = r.read_string()?;
        cfg.fields.txt_domain_string = r.read_string()?;
        cfg.fields.reaction_default_emoji = r.read_string()?;
        cfg.fields.reaction_default_custom = r.read_u64()?;
        Some(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_and_roundtrip() {
        let mut cfg = MtpConfig::new(Environment::Production);
        assert!(!cfg.is_test_mode());
        assert_eq!(cfg.fields.web_file_dc_id, 4);
        assert_eq!(cfg.fields.caption_length_max, 1024);

        cfg.fields.caption_length_max = 2048;
        cfg.fields.reaction_default_emoji = "❤️".to_string();

        let raw = cfg.serialize();
        let decoded = MtpConfig::from_serialized(Environment::Production, &raw).unwrap();
        assert_eq!(decoded.fields.caption_length_max, 2048);
        assert_eq!(decoded.fields.reaction_default_emoji, "❤️");
    }
}
