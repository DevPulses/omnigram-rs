//! Verification code validation and `auth.signIn` / `auth.resendCode` requests.
//!
//! Source: `tdesktop/Telegram/SourceFiles/intro/intro_code.{h,cpp}`.

/// Request payload corresponding to `MTPauth_SignIn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInRequest {
    pub phone_number: String,
    pub phone_code_hash: String,
    pub phone_code: String,
}

impl SignInRequest {
    pub fn new(
        phone_number: impl Into<String>,
        phone_code_hash: impl Into<String>,
        phone_code: impl Into<String>,
    ) -> Self {
        Self {
            phone_number: phone_number.into(),
            phone_code_hash: phone_code_hash.into(),
            phone_code: phone_code.into(),
        }
    }
}

/// Request payload corresponding to `MTPauth_ResendCode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResendCodeRequest {
    pub phone_number: String,
    pub phone_code_hash: String,
}

/// Validates entered verification code against expected `code_length`.
pub struct CodeInputValidator;

impl CodeInputValidator {
    pub fn sanitize(raw: &str, max_len: usize) -> String {
        raw.chars()
            .filter(|c| c.is_ascii_digit())
            .take(max_len)
            .collect()
    }

    pub fn is_complete(code: &str, expected_len: usize) -> bool {
        code.len() == expected_len && code.bytes().all(|b| b.is_ascii_digit())
    }
}
