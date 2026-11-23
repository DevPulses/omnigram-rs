//! `MTP::Environment` — which server fleet the options describe.

/// `MTP::Environment`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    Production,
    Test,
}

impl Environment {
    /// `isTestMode`.
    #[must_use]
    pub const fn is_test(self) -> bool {
        !matches!(self, Self::Production)
    }
}
