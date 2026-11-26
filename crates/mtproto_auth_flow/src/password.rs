//! Cloud password (2FA) state and recovery requests.
//!
//! Source: `tdesktop/Telegram/SourceFiles/intro/intro_password_check.{h,cpp}`.

/// Cloud password (2FA) parameters matching `Core::CloudPasswordState`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CloudPasswordState {
    pub has_password: bool,
    pub has_recovery: bool,
    pub hint: String,
    pub srp_id: u64,
    pub srp_b: Vec<u8>,
    pub random_salt: Vec<u8>,
    pub email_unconfirmed_pattern: String,
}

/// Request payload corresponding to `MTPauth_CheckPassword`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckPasswordRequest {
    pub srp_id: u64,
    pub a: Vec<u8>,
    pub m1: Vec<u8>,
}

/// Request payload corresponding to `MTPauth_RecoverPassword`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRequest {
    pub code: String,
}
