//! Account and session lifecycle management matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/main/main_account.{h,cpp}` and `main_session.{h,cpp}`).

use models_peers::id::UserId;
use session_settings::SessionSettings;
use mtproto_session::MtprotoSession;
use mtproto_auth_key::AuthKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountState {
    LoggedOut,
    Connecting,
    Authorized,
}

pub struct Session {
    user_id: UserId,
    settings: SessionSettings,
    mtproto: MtprotoSession,
}

impl Session {
    pub fn new(user_id: UserId, settings: SessionSettings, mtproto: MtprotoSession) -> Self {
        Self {
            user_id,
            settings,
            mtproto,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn settings(&self) -> &SessionSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut SessionSettings {
        &mut self.settings
    }

    pub fn mtproto(&self) -> &MtprotoSession {
        &self.mtproto
    }

    pub fn mtproto_mut(&mut self) -> &mut MtprotoSession {
        &mut self.mtproto
    }
}

pub struct Account {
    index: usize,
    data_name: String,
    state: AccountState,
    session: Option<Session>,
}

impl Account {
    pub fn new(index: usize, data_name: impl Into<String>) -> Self {
        Self {
            index,
            data_name: data_name.into(),
            state: AccountState::LoggedOut,
            session: None,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn data_name(&self) -> &str {
        &self.data_name
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn is_authorized(&self) -> bool {
        self.state == AccountState::Authorized && self.session.is_some()
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    pub fn session_mut(&mut self) -> Option<&mut Session> {
        self.session.as_mut()
    }

    pub fn create_session(&mut self, user_id: UserId, auth_key: AuthKey, server_salt: u64, settings: Option<SessionSettings>) {
        let mtproto = MtprotoSession::new(auth_key, server_salt);
        let settings = settings.unwrap_or_default();
        self.session = Some(Session::new(user_id, settings, mtproto));
        self.state = AccountState::Authorized;
    }

    pub fn log_out(&mut self) {
        self.session = None;
        self.state = AccountState::LoggedOut;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation_and_logout() {
        let mut account = Account::new(0, "data");
        assert_eq!(account.state(), AccountState::LoggedOut);
        assert!(!account.is_authorized());

        let user_id = UserId::new(123456);
        let auth_key = AuthKey::from_key([0x42; 256]);
        account.create_session(user_id, auth_key, 987654, None);

        assert_eq!(account.state(), AccountState::Authorized);
        assert!(account.is_authorized());
        assert_eq!(account.session().unwrap().user_id(), user_id);

        account.log_out();
        assert_eq!(account.state(), AccountState::LoggedOut);
        assert!(!account.is_authorized());
        assert!(account.session().is_none());
    }
}
