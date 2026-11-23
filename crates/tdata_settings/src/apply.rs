//! `ApplyReadFallbackConfig` — what the pass decided to do with the fallback
//! config pieces.
//!
//! Source of truth: `ApplyReadFallbackConfig` in
//! `storage/details/storage_settings_scheme.cpp`.
//!
//! The C++ function writes into `Core::App().fallbackProductionConfig()`; here
//! the decision travels back to the caller, because the config store belongs to
//! the core area.

use crate::context::{LegacyDcOptions, ReadSettingsContext};

/// The legacy pieces to merge into the production fallback config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFallbackConfig {
    /// `dcOptions().addFromOther(...)` — the endpoints or the blob.
    pub dc_options: LegacyDcOptions,
    /// `setChatSizeMax` — only when the read value was positive.
    pub chat_size_max: Option<i32>,
    /// `setStickersRecentLimit` — only when positive.
    pub stickers_recent_limit: Option<i32>,
    /// `setMegagroupSizeMax` — only when positive.
    pub megagroup_size_max: Option<i32>,
    /// `setTxtDomainString` — only when non-empty.
    pub txt_domain_string: Option<String>,
}

/// What to do with the production fallback config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackConfigAction {
    /// No serialized config was stored: merge these legacy pieces into the
    /// built-in one.
    Legacy(LegacyFallbackConfig),
    /// `constructFallbackProductionConfig` — rebuild the config from these
    /// bytes instead of merging anything.
    Construct(Vec<u8>),
}

/// `ApplyReadFallbackConfig` — decides between merging legacy pieces and
/// rebuilding the config from a serialized blob.
///
/// Takes the legacy dc options out of the context, as the C++ code moves them.
#[must_use]
pub fn apply_read_fallback_config(context: &mut ReadSettingsContext) -> FallbackConfigAction {
    if context.fallback_config.is_empty() {
        let positive = |value: i32| (value > 0).then_some(value);
        let txt_domain_string = (!context.fallback_config_legacy_txt_domain_string.is_empty())
            .then(|| std::mem::take(&mut context.fallback_config_legacy_txt_domain_string));
        return FallbackConfigAction::Legacy(LegacyFallbackConfig {
            dc_options: std::mem::take(&mut context.fallback_config_legacy_dc_options),
            chat_size_max: positive(context.fallback_config_legacy_chat_size_max),
            stickers_recent_limit: positive(context.fallback_config_legacy_stickers_recent_limit),
            megagroup_size_max: positive(context.fallback_config_legacy_megagroup_size_max),
            txt_domain_string,
        });
    }
    FallbackConfigAction::Construct(std::mem::take(&mut context.fallback_config))
}