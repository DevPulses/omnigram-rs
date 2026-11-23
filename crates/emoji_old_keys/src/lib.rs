//! Legacy emoji old-key conversion functions matching Telegram Desktop
//! (`tdesktop/Telegram/lib_ui/ui/emoji_config.{h,cpp}` and
//! `tdesktop/Telegram/SourceFiles/storage/details/storage_settings_scheme.cpp`).

/// Converts an old 64-bit emoji key to a Unicode UTF-8 string id.
///
/// Ported from `QString Ui::Emoji::IdFromOldKey(uint64 oldKey)`:
/// ```cpp
/// QString IdFromOldKey(uint64 oldKey) {
///     auto code = uint32(oldKey >> 32);
///     auto code2 = uint32(oldKey & 0xFFFFFFFFLLU);
///     if (!code && code2) {
///         code = base::take(code2);
///     }
///     if ((code & 0xFFFF0000U) != 0xFFFF0000U) { // code and code2 contain the whole id
///         auto result = QString();
///         result.reserve(4);
///         auto addCode = [&result](uint32 code) {
///             if (auto high = (code >> 16)) {
///                 result.append(QChar(static_cast<ushort>(high & 0xFFFFU)));
///             }
///             result.append(QChar(static_cast<ushort>(code & 0xFFFFU)));
///         };
///         addCode(code);
///         if (code2) addCode(code2);
///         return result;
///     }
/// ...
/// ```
pub fn id_from_old_key(old_key: u64) -> Option<String> {
    let mut code = (old_key >> 32) as u32;
    let mut code2 = (old_key & 0xFFFF_FFFF) as u32;

    if code == 0 && code2 != 0 {
        code = code2;
        code2 = 0;
    }

    if (code & 0xFFFF_0000) != 0xFFFF_0000 {
        // code and code2 contain UTF-16 code units
        let mut utf16 = Vec::with_capacity(4);
        let mut add_code = |c: u32| {
            let high = (c >> 16) as u16;
            if high != 0 {
                utf16.push(high);
            }
            utf16.push((c & 0xFFFF) as u16);
        };
        add_code(code);
        if code2 != 0 {
            add_code(code2);
        }
        return String::from_utf16(&utf16).ok();
    }

    // Old predefined sequences
    let sequence_index = (code & 0xFFFF) as i32;
    let s = match sequence_index {
        0 => "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
        1 => "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
        2 => "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F466}\u{200D}\u{1F466}",
        3 => "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F467}",
        4 => "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F466}",
        5 => "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}",
        6 => "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
        7 => "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F466}\u{200D}\u{1F466}",
        8 => "\u{1F469}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F467}",
        9 => "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F466}",
        10 => "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}",
        11 => "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F466}",
        12 => "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F466}\u{200D}\u{1F466}",
        13 => "\u{1F468}\u{200D}\u{1F468}\u{200D}\u{1F467}\u{200D}\u{1F467}",
        14 => "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F469}",
        15 => "\u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468}",
        16 => "\u{1F469}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F48B}\u{200D}\u{1F469}",
        17 => "\u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F48B}\u{200D}\u{1F468}",
        18 => "\u{1F441}\u{200D}\u{1F5E8}",
        _ => return None,
    };
    Some(s.to_string())
}

/// Remaps legacy flag emoji keys from `storage_settings_scheme.cpp`.
pub fn fix_legacy_flag_key(old_key: u64) -> u64 {
    match old_key {
        0xD83C_DDEF => 0xD83C_DDEF_D83C_DDF5,
        0xD83C_DDF0 => 0xD83C_DDF0_D83C_DDF7,
        0xD83C_DDE9 => 0xD83C_DDE9_D83C_DDEA,
        0xD83C_DDE8 => 0xD83C_DDE8_D83C_DDF3,
        0xD83C_DDFA => 0xD83C_DDFA_D83C_DDF8,
        0xD83C_DDEB => 0xD83C_DDEB_D83C_DDF7,
        0xD83C_DDEA => 0xD83C_DDEA_D83C_DDF8,
        0xD83C_DDEE => 0xD83C_DDEE_D83C_DDF9,
        0xD83C_DDF7 => 0xD83C_DDF7_D83C_DDFA,
        0xD83C_DDEC => 0xD83C_DDEC_D83C_DDE7,
        other => other,
    }
}

/// Color index from skin tone modifier code.
///
/// Ported from `int Ui::Emoji::ColorIndexFromCode(uint32 code)`:
/// - 0xD83CDFFB => 1 (light)
/// - 0xD83CDFFC => 2 (medium-light)
/// - 0xD83CDFFD => 3 (medium)
/// - 0xD83CDFFE => 4 (medium-dark)
/// - 0xD83CDFFF => 5 (dark)
pub fn color_index_from_code(code: u32) -> i32 {
    match code {
        0xD83C_DFFB => 1,
        0xD83C_DFFC => 2,
        0xD83C_DFFD => 3,
        0xD83C_DFFE => 4,
        0xD83C_DFFF => 5,
        _ => 0,
    }
}

/// Color index from old emoji key.
pub fn color_index_from_old_key(old_key: u64) -> i32 {
    color_index_from_code((old_key & 0xFFFF_FFFF) as u32)
}

/// Default recent emoji old keys list from `Ui::Emoji::GetDefaultRecent()`.
pub const DEFAULT_RECENT_OLD_KEYS: &[u64] = &[
    0xD83D_DE02,
    0xD83D_DE18,
    0x2764,
    0xD83D_DE0D,
    0xD83D_DE0A,
    0xD83D_DE01,
    0xD83D_DC4D,
    0x263A,
    0xD83D_DE14,
    0xD83D_DE04,
    0xD83D_DE2D,
    0xD83D_DC8B,
    0xD83D_DE12,
    0xD83D_DE33,
    0xD83D_DE1C,
    0xD83D_DE48,
    0xD83D_DE09,
    0xD83D_DE03,
    0xD83D_DE22,
    0xD83D_DE1D,
    0xD83D_DE31,
    0xD83D_DE21,
    0xD83D_DE0F,
    0xD83D_DE1E,
    0xD83D_DE05,
    0xD83D_DE1A,
    0xD83D_DE4A,
    0xD83D_DE0C,
    0xD83D_DE00,
    0xD83D_DE0B,
    0xD83D_DE06,
    0xD83D_DC4C,
    0xD83D_DE10,
    0xD83D_DE15,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_codepoint() {
        // 0x2764 is Red Heart (❤)
        let s = id_from_old_key(0x2764).expect("should convert");
        assert_eq!(s, "❤");
    }

    #[test]
    fn test_surrogate_pair() {
        // 0xD83DDE02 is Face with Tears of Joy (😂)
        // High surrogate 0xD83D, Low surrogate 0xDE02
        let s = id_from_old_key(0xD83D_DE02).expect("should convert");
        assert_eq!(s, "😂");
    }

    #[test]
    fn test_combined_surrogates() {
        // Thumbs up (👍 = 0xD83D_DC4D) + Medium-Light Skin Tone (0xD83C_DFFC)
        let key = ((0xD83D_DC4D as u64) << 32) | (0xD83C_DFFC as u64);
        let s = id_from_old_key(key).expect("should convert");
        assert_eq!(s, "👍🏼");
        assert_eq!(color_index_from_old_key(key), 2);
    }

    #[test]
    fn test_sequence_indices() {
        // Family sequence 0
        let key = (0xFFFF_0000_u64) << 32;
        let s = id_from_old_key(key).expect("sequence 0");
        assert_eq!(s, "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}");
    }

    #[test]
    fn test_fix_legacy_flag_key() {
        assert_eq!(fix_legacy_flag_key(0xD83C_DDEF), 0xD83C_DDEF_D83C_DDF5);
        let id = id_from_old_key(fix_legacy_flag_key(0xD83C_DDEF)).expect("flag");
        assert!(!id.is_empty());
    }

    #[test]
    fn test_default_recent_all_valid() {
        for &k in DEFAULT_RECENT_OLD_KEYS {
            let res = id_from_old_key(k);
            assert!(res.is_some(), "Key 0x{:X} should parse", k);
        }
    }
}
