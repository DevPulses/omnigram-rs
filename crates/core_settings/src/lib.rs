//! `core_settings` — Application-wide Core Settings ported from Telegram Desktop.
//!
//! Source:
//! - `tdesktop/Telegram/SourceFiles/core/core_settings.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/core/core_settings_proxy.{h,cpp}`

pub mod types;
pub mod window_position;
pub mod settings;

pub use types::*;
pub use window_position::*;
pub use settings::*;
