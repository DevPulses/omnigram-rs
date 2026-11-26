//! MTProto user authentication state machine (phone, code, 2FA password).
//!
//! Ported from `tdesktop/Telegram/SourceFiles/intro/intro_{widget,phone,code,password_check}.{h,cpp}`.

pub mod code;
pub mod password;
pub mod phone;
pub mod state;

pub use code::{CodeInputValidator, ResendCodeRequest, SignInRequest};
pub use password::{CheckPasswordRequest, CloudPasswordState, RecoveryRequest};
pub use phone::{is_valid_phone_digits, normalize_phone, SendCodeRequest};
pub use state::{
    AuthError, AuthFlowData, AuthFlowMachine, AuthStep, CallStatus, EmailStatus, EnterPoint,
    SentCodeType,
};
