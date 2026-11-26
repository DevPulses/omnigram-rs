//! Authentication flow state machine matching `Intro::Widget` and `Intro::details::Data`.

use std::fmt;

use crate::code::{CodeInputValidator, ResendCodeRequest, SignInRequest};
use crate::password::{CheckPasswordRequest, CloudPasswordState};
use crate::phone::{is_valid_phone_digits, normalize_phone, SendCodeRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterPoint {
    Start,
    Phone,
    Qr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStep {
    Start,
    PhoneInput,
    CodeVerify,
    Password2FA,
    SignUp,
    Authorized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallStatus {
    Waiting,
    Calling,
    Called,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailStatus {
    None,
    SetupRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentCodeType {
    App { length: usize },
    Sms { length: usize },
    Call { length: usize },
    FlashCall { pattern: String },
    FragmentSms { url: String, length: usize },
    EmailCode { email_pattern: String, length: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    InvalidPhone,
    InvalidCode,
    InvalidStepTransition,
    MissingPasswordState,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPhone => write!(f, "Invalid phone number"),
            Self::InvalidCode => write!(f, "Invalid verification code"),
            Self::InvalidStepTransition => write!(f, "Invalid auth flow step transition"),
            Self::MissingPasswordState => write!(f, "Missing 2FA cloud password state"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Shared intro state matching `Intro::details::Data`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthFlowData {
    pub country: String,
    pub phone: String,
    pub phone_hash: String,
    pub call_status: CallStatus,
    pub call_timeout: i32,
    pub code_length: usize,
    pub code_by_telegram: bool,
    pub code_by_fragment_url: Option<String>,
    pub email_status: EmailStatus,
    pub email: String,
    pub email_pattern_setup: String,
    pub email_pattern_login: String,
    pub pwd_state: CloudPasswordState,
}

impl Default for AuthFlowData {
    fn default() -> Self {
        Self {
            country: String::new(),
            phone: String::new(),
            phone_hash: String::new(),
            call_status: CallStatus::Disabled,
            call_timeout: 0,
            code_length: 5,
            code_by_telegram: false,
            code_by_fragment_url: None,
            email_status: EmailStatus::None,
            email: String::new(),
            email_pattern_setup: String::new(),
            email_pattern_login: String::new(),
            pwd_state: CloudPasswordState::default(),
        }
    }
}

/// Finite state machine driving the Telegram login flow.
#[derive(Debug, Clone)]
pub struct AuthFlowMachine {
    step: AuthStep,
    history: Vec<AuthStep>,
    pub data: AuthFlowData,
}

impl AuthFlowMachine {
    pub fn new(entry: EnterPoint) -> Self {
        let step = match entry {
            EnterPoint::Start | EnterPoint::Qr => AuthStep::Start,
            EnterPoint::Phone => AuthStep::PhoneInput,
        };
        Self {
            step,
            history: Vec::new(),
            data: AuthFlowData::default(),
        }
    }

    pub fn step(&self) -> AuthStep {
        self.step
    }

    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn go_back(&mut self) -> Option<AuthStep> {
        let prev = self.history.pop()?;
        self.step = prev;
        Some(prev)
    }

    pub fn begin_phone_entry(&mut self) {
        if self.step == AuthStep::Start {
            self.history.push(self.step);
            self.step = AuthStep::PhoneInput;
        }
    }

    pub fn submit_phone(
        &mut self,
        raw_phone: &str,
        api_id: i32,
        api_hash: &str,
    ) -> Result<SendCodeRequest, AuthError> {
        if self.step != AuthStep::PhoneInput && self.step != AuthStep::Start {
            return Err(AuthError::InvalidStepTransition);
        }
        let normalized = normalize_phone(raw_phone);
        if !is_valid_phone_digits(&normalized) {
            return Err(AuthError::InvalidPhone);
        }
        self.data.phone = normalized.clone();
        Ok(SendCodeRequest::new(normalized, api_id, api_hash))
    }

    pub fn on_code_sent(
        &mut self,
        phone_code_hash: String,
        sent_type: SentCodeType,
        next_timeout_secs: Option<i32>,
    ) {
        self.data.phone_hash = phone_code_hash;
        self.data.code_by_telegram = false;
        self.data.code_by_fragment_url = None;

        match sent_type {
            SentCodeType::App { length } => {
                self.data.code_length = length;
                self.data.code_by_telegram = true;
            }
            SentCodeType::Sms { length } | SentCodeType::Call { length } => {
                self.data.code_length = length;
            }
            SentCodeType::FlashCall { .. } => {
                self.data.code_length = 5;
            }
            SentCodeType::FragmentSms { url, length } => {
                self.data.code_length = length;
                self.data.code_by_fragment_url = Some(url);
            }
            SentCodeType::EmailCode {
                email_pattern,
                length,
            } => {
                self.data.code_length = length;
                self.data.email_pattern_login = email_pattern;
            }
        }

        if let Some(secs) = next_timeout_secs {
            if secs > 0 {
                self.data.call_status = CallStatus::Waiting;
                self.data.call_timeout = secs;
            } else {
                self.data.call_status = CallStatus::Disabled;
                self.data.call_timeout = 0;
            }
        } else {
            self.data.call_status = CallStatus::Disabled;
            self.data.call_timeout = 0;
        }

        if self.step != AuthStep::CodeVerify {
            self.history.push(self.step);
            self.step = AuthStep::CodeVerify;
        }
    }

    pub fn tick_call_timer(&mut self) -> CallStatus {
        if self.data.call_status == CallStatus::Waiting && self.data.call_timeout > 0 {
            self.data.call_timeout -= 1;
            if self.data.call_timeout == 0 {
                self.data.call_status = CallStatus::Calling;
            }
        }
        self.data.call_status
    }

    pub fn request_resend(&self) -> Result<ResendCodeRequest, AuthError> {
        if self.step != AuthStep::CodeVerify {
            return Err(AuthError::InvalidStepTransition);
        }
        Ok(ResendCodeRequest {
            phone_number: self.data.phone.clone(),
            phone_code_hash: self.data.phone_hash.clone(),
        })
    }

    pub fn submit_code(&self, raw_code: &str) -> Result<SignInRequest, AuthError> {
        if self.step != AuthStep::CodeVerify {
            return Err(AuthError::InvalidStepTransition);
        }
        let code = CodeInputValidator::sanitize(raw_code, self.data.code_length);
        if !CodeInputValidator::is_complete(&code, self.data.code_length) {
            return Err(AuthError::InvalidCode);
        }
        Ok(SignInRequest::new(
            self.data.phone.clone(),
            self.data.phone_hash.clone(),
            code,
        ))
    }

    pub fn on_password_required(&mut self, pwd_state: CloudPasswordState) -> Result<(), AuthError> {
        if self.step != AuthStep::CodeVerify {
            return Err(AuthError::InvalidStepTransition);
        }
        self.data.pwd_state = pwd_state;
        self.history.push(self.step);
        self.step = AuthStep::Password2FA;
        Ok(())
    }

    pub fn on_sign_up_required(&mut self) -> Result<(), AuthError> {
        if self.step != AuthStep::CodeVerify {
            return Err(AuthError::InvalidStepTransition);
        }
        self.history.push(self.step);
        self.step = AuthStep::SignUp;
        Ok(())
    }

    pub fn prepare_password_check(
        &self,
        a: Vec<u8>,
        m1: Vec<u8>,
    ) -> Result<CheckPasswordRequest, AuthError> {
        if self.step != AuthStep::Password2FA {
            return Err(AuthError::InvalidStepTransition);
        }
        if !self.data.pwd_state.has_password || self.data.pwd_state.srp_id == 0 {
            return Err(AuthError::MissingPasswordState);
        }
        Ok(CheckPasswordRequest {
            srp_id: self.data.pwd_state.srp_id,
            a,
            m1,
        })
    }

    pub fn on_authorized(&mut self) {
        self.history.clear();
        self.step = AuthStep::Authorized;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_auth_flow_with_2fa() {
        let mut machine = AuthFlowMachine::new(EnterPoint::Phone);
        assert_eq!(machine.step(), AuthStep::PhoneInput);

        // Invalid short phone rejected
        assert_eq!(
            machine.submit_phone("12", 12345, "hash"),
            Err(AuthError::InvalidPhone)
        );

        let send_req = machine.submit_phone("+1 (555) 123-4567", 12345, "hash").unwrap();
        assert_eq!(send_req.phone_number, "15551234567");

        machine.on_code_sent(
            "abc_hash".to_string(),
            SentCodeType::App { length: 5 },
            Some(2),
        );
        assert_eq!(machine.step(), AuthStep::CodeVerify);
        assert!(machine.data.code_by_telegram);
        assert_eq!(machine.data.call_status, CallStatus::Waiting);

        assert_eq!(machine.tick_call_timer(), CallStatus::Waiting);
        assert_eq!(machine.tick_call_timer(), CallStatus::Calling);

        assert_eq!(machine.submit_code("123"), Err(AuthError::InvalidCode));
        let sign_in = machine.submit_code("54321").unwrap();
        assert_eq!(sign_in.phone_code, "54321");
        assert_eq!(sign_in.phone_code_hash, "abc_hash");

        machine
            .on_password_required(CloudPasswordState {
                has_password: true,
                has_recovery: true,
                hint: "my cat".to_string(),
                srp_id: 999,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(machine.step(), AuthStep::Password2FA);

        let pwd_req = machine.prepare_password_check(vec![1, 2], vec![3, 4]).unwrap();
        assert_eq!(pwd_req.srp_id, 999);

        machine.on_authorized();
        assert_eq!(machine.step(), AuthStep::Authorized);
        assert!(!machine.can_go_back());
    }

    #[test]
    fn test_back_navigation() {
        let mut machine = AuthFlowMachine::new(EnterPoint::Start);
        machine.begin_phone_entry();
        assert_eq!(machine.step(), AuthStep::PhoneInput);
        assert!(machine.can_go_back());
        assert_eq!(machine.go_back(), Some(AuthStep::Start));
        assert_eq!(machine.step(), AuthStep::Start);
    }
}
