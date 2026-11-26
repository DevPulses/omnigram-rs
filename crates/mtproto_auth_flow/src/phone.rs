//! Phone number normalization and `auth.sendCode` request builder.
//!
//! Source: `tdesktop/Telegram/SourceFiles/intro/intro_phone.{h,cpp}`.

/// Normalizes a phone string by keeping only ASCII digits (stripping `+`, spaces, `-`, `(`, `)`).
pub fn normalize_phone(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Validates that normalized phone digits have a plausible E.164 length (4..=15 digits).
pub fn is_valid_phone_digits(digits: &str) -> bool {
    let len = digits.len();
    (4..=15).contains(&len) && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Request payload corresponding to `MTPauth_SendCode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendCodeRequest {
    pub phone_number: String,
    pub api_id: i32,
    pub api_hash: String,
    pub allow_flashcall: bool,
    pub current_number: bool,
    pub allow_app_hash: bool,
}

impl SendCodeRequest {
    pub fn new(phone_number: impl Into<String>, api_id: i32, api_hash: impl Into<String>) -> Self {
        Self {
            phone_number: normalize_phone(&phone_number.into()),
            api_id,
            api_hash: api_hash.into(),
            allow_flashcall: false,
            current_number: true,
            allow_app_hash: true,
        }
    }
}
