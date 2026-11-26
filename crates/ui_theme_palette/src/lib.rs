//! Window theme manager and runtime color palette switching.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/window/themes/window_theme.{h,cpp}`.

use styles_parser::palette::Palette;
use styles_parser::types::Color;

pub const THEME_FILE_SIZE_LIMIT: usize = 5 * 1024 * 1024;
pub const THEME_SCHEME_SIZE_LIMIT: usize = 1024 * 1024;
pub const THEME_BACKGROUND_SIZE_LIMIT: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmbeddedTheme {
    Default,
    Day,
    DayBlue,
    Night,
    NightGreen,
}

impl EmbeddedTheme {
    pub fn from_path(path: &str) -> Option<Self> {
        match path {
            ":/gui/day-custom.tdesktop-theme" => Some(Self::Default),
            ":/gui/day.tdesktop-theme" => Some(Self::Day),
            ":/gui/day-blue.tdesktop-theme" => Some(Self::DayBlue),
            ":/gui/night.tdesktop-theme" => Some(Self::Night),
            ":/gui/night-green.tdesktop-theme" => Some(Self::NightGreen),
            _ => None,
        }
    }

    pub fn is_night(&self) -> bool {
        matches!(self, Self::Night | Self::NightGreen)
    }

    pub fn path(&self) -> &'static str {
        match self {
            Self::Default => ":/gui/day-custom.tdesktop-theme",
            Self::Day => ":/gui/day.tdesktop-theme",
            Self::DayBlue => ":/gui/day-blue.tdesktop-theme",
            Self::Night => ":/gui/night.tdesktop-theme",
            Self::NightGreen => ":/gui/night-green.tdesktop-theme",
        }
    }
}

pub fn is_embedded_theme(path: &str) -> bool {
    EmbeddedTheme::from_path(path).is_some()
}

#[derive(Debug, Clone, Default)]
pub struct ThemeCached {
    pub palette_checksum: i32,
    pub content_checksum: i32,
    pub tiled: bool,
}

/// Manages active palette, fallback color resolution, and night mode toggling.
#[derive(Debug, Clone)]
pub struct ThemeManager {
    palette: Palette,
    night_mode: bool,
    active_path: String,
    cached: ThemeCached,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            palette: Palette::default(),
            night_mode: false,
            active_path: EmbeddedTheme::Default.path().to_string(),
            cached: ThemeCached::default(),
        }
    }

    pub fn apply_palette_text(&mut self, palette_src: &str) -> Result<(), String> {
        if palette_src.len() > THEME_SCHEME_SIZE_LIMIT {
            return Err("Theme scheme exceeds size limit".to_string());
        }
        self.palette = Palette::parse(palette_src)?;
        Ok(())
    }

    pub fn set_night_mode(&mut self, night: bool) {
        self.night_mode = night;
        if night && !is_embedded_theme(&self.active_path) {
            self.active_path = EmbeddedTheme::Night.path().to_string();
        } else if night {
            self.active_path = EmbeddedTheme::Night.path().to_string();
        } else {
            self.active_path = EmbeddedTheme::Default.path().to_string();
        }
    }

    pub fn is_night_mode(&self) -> bool {
        self.night_mode
    }

    pub fn active_path(&self) -> &str {
        &self.active_path
    }

    pub fn cached(&self) -> &ThemeCached {
        &self.cached
    }

    /// Resolves a named color from the palette, following fallback chains up to 8 hops.
    pub fn resolve_color(&self, name: &str) -> Option<Color> {
        let mut current = self.palette.colors.get(name)?;
        for _ in 0..8 {
            if let Some(ref fb) = current.fallback {
                if let Some(next) = self.palette.colors.get(fb) {
                    if current.r == 0 && current.g == 0 && current.b == 0 && current.a == 255 {
                        current = next;
                        continue;
                    }
                }
            }
            break;
        }
        Some(current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_themes_and_night_mode() {
        assert!(is_embedded_theme(":/gui/night.tdesktop-theme"));
        assert!(!is_embedded_theme("/tmp/custom.tdesktop-theme"));

        let mut mgr = ThemeManager::new();
        assert!(!mgr.is_night_mode());
        mgr.set_night_mode(true);
        assert!(mgr.is_night_mode());
        assert_eq!(mgr.active_path(), EmbeddedTheme::Night.path());
    }

    #[test]
    fn test_palette_parse_and_resolve() {
        let mut mgr = ThemeManager::new();
        let src = "
            windowBg: #ffffff;
            windowFg: #000000;
            dialogsBg: windowBg;
            msgOutBg: #effdde;
        ";
        mgr.apply_palette_text(src).unwrap();

        let bg = mgr.resolve_color("windowBg").unwrap();
        assert_eq!((bg.r, bg.g, bg.b, bg.a), (255, 255, 255, 255));

        let dialogs_bg = mgr.resolve_color("dialogsBg").unwrap();
        assert_eq!((dialogs_bg.r, dialogs_bg.g, dialogs_bg.b), (255, 255, 255));
    }
}
