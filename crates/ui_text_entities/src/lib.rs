//! Rich text entities, UTF-16 offset mapping, and inline entity parsing.
//!
//! Ported from `tdesktop/Telegram/lib_ui/ui/text/text_entity.{h,cpp}`
//! and `tdesktop/Telegram/lib_ui/ui/text/text.{h,cpp}`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Invalid,
    Url,
    CustomUrl,
    Email,
    Hashtag,
    Cashtag,
    Mention,
    MentionName,
    CustomEmoji,
    BotCommand,
    MediaTimestamp,
    Colorized,
    Phone,
    BankCard,
    TonAddress,
    Bold,
    Semibold,
    Italic,
    Underline,
    StrikeOut,
    Code,
    Pre,
    Blockquote,
    Spoiler,
    Subscript,
    Superscript,
    Marked,
    FormattedDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityInText {
    pub entity_type: EntityType,
    pub offset: usize,
    pub length: usize,
    pub data: String,
}

impl EntityInText {
    pub fn new(
        entity_type: EntityType,
        offset: usize,
        length: usize,
        data: impl Into<String>,
    ) -> Self {
        Self {
            entity_type,
            offset,
            length,
            data: data.into(),
        }
    }

    /// Extracts the substring covered by this entity using UTF-16 code unit offsets
    /// (matching Telegram MTProto / Qt `QString` entity offsets).
    pub fn extract_from_utf16(&self, text: &str) -> Option<String> {
        let units: Vec<u16> = text.encode_utf16().collect();
        let end = self.offset.checked_add(self.length)?;
        if end > units.len() {
            return None;
        }
        String::from_utf16(&units[self.offset..end]).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextWithEntities {
    pub text: String,
    pub entities: Vec<EntityInText>,
}

impl TextWithEntities {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            entities: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Scans plain text for `@mentions`, `#hashtags`, `/bot_commands`, and `http(s)://` URLs,
    /// recording accurate UTF-16 offsets.
    pub fn parse_plain(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut entities = Vec::new();
        let mut utf16_pos = 0usize;
        let mut byte_idx = 0usize;
        let bytes = text.as_bytes();

        while byte_idx < bytes.len() {
            let ch = text[byte_idx..].chars().next().unwrap();
            let ch_utf8 = ch.len_utf8();
            let ch_utf16 = ch.len_utf16();

            let is_boundary = byte_idx == 0
                || text[..byte_idx]
                    .chars()
                    .next_back()
                    .map(|p| p.is_whitespace())
                    .unwrap_or(true);

            if is_boundary && (ch == '@' || ch == '#' || ch == '/') {
                let start_u16 = utf16_pos;
                let mut cur_byte = byte_idx + ch_utf8;
                let mut cur_u16 = utf16_pos + ch_utf16;

                while cur_byte < bytes.len() {
                    let next_ch = text[cur_byte..].chars().next().unwrap();
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        cur_byte += next_ch.len_utf8();
                        cur_u16 += next_ch.len_utf16();
                    } else {
                        break;
                    }
                }

                if cur_u16 > start_u16 + 1 {
                    let kind = match ch {
                        '@' => EntityType::Mention,
                        '#' => EntityType::Hashtag,
                        '/' => EntityType::BotCommand,
                        _ => unreachable!(),
                    };
                    entities.push(EntityInText::new(kind, start_u16, cur_u16 - start_u16, ""));
                    byte_idx = cur_byte;
                    utf16_pos = cur_u16;
                    continue;
                }
            } else if is_boundary
                && (text[byte_idx..].starts_with("https://")
                    || text[byte_idx..].starts_with("http://"))
            {
                let start_u16 = utf16_pos;
                let mut cur_byte = byte_idx;
                let mut cur_u16 = utf16_pos;
                while cur_byte < bytes.len() {
                    let next_ch = text[cur_byte..].chars().next().unwrap();
                    if next_ch.is_whitespace() {
                        break;
                    }
                    cur_byte += next_ch.len_utf8();
                    cur_u16 += next_ch.len_utf16();
                }
                entities.push(EntityInText::new(
                    EntityType::Url,
                    start_u16,
                    cur_u16 - start_u16,
                    "",
                ));
                byte_idx = cur_byte;
                utf16_pos = cur_u16;
                continue;
            }

            byte_idx += ch_utf8;
            utf16_pos += ch_utf16;
        }

        Self { text, entities }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf16_entity_offsets_with_emoji() {
        // "🔥 @rust_lang #dev" -> 🔥 is 2 UTF-16 code units!
        let parsed = TextWithEntities::parse_plain("🔥 @rust_lang #dev https://t.me/test");
        assert_eq!(parsed.entities.len(), 3);

        assert_eq!(parsed.entities[0].entity_type, EntityType::Mention);
        assert_eq!(parsed.entities[0].offset, 3); // 2 (emoji) + 1 (space)
        assert_eq!(
            parsed.entities[0].extract_from_utf16(&parsed.text).as_deref(),
            Some("@rust_lang")
        );

        assert_eq!(parsed.entities[1].entity_type, EntityType::Hashtag);
        assert_eq!(
            parsed.entities[1].extract_from_utf16(&parsed.text).as_deref(),
            Some("#dev")
        );

        assert_eq!(parsed.entities[2].entity_type, EntityType::Url);
        assert_eq!(
            parsed.entities[2].extract_from_utf16(&parsed.text).as_deref(),
            Some("https://t.me/test")
        );
    }
}
