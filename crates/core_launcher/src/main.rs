//! Omnigram-rs application entry point and launcher matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/main.cpp` and `core/launcher.{h,cpp}`).

use std::path::{Path, PathBuf};
use core_settings::settings::Settings as CoreSettings;
use main_account_session::Account;
use ui_main_window::WindowLayout;

pub struct Launcher {
    working_dir: PathBuf,
    arguments: Vec<String>,
    settings: CoreSettings,
    layout: WindowLayout,
    account: Account,
}

impl Launcher {
    pub fn new(args: Vec<String>) -> Self {
        let mut working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut custom_dir = None;

        let mut i = 1;
        while i < args.len() {
            if args[i] == "-workdir" && i + 1 < args.len() {
                custom_dir = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            i += 1;
        }

        if let Some(dir) = custom_dir {
            working_dir = dir;
        }

        Self {
            working_dir,
            arguments: args,
            settings: CoreSettings::default(),
            layout: WindowLayout::default(),
            account: Account::new(0, "data"),
        }
    }

    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn settings(&self) -> &CoreSettings {
        &self.settings
    }

    pub fn layout(&self) -> &WindowLayout {
        &self.layout
    }

    pub fn account(&self) -> &Account {
        &self.account
    }

    pub fn run(&self) -> i32 {
        println!("Starting Omnigram-rs (Telegram Desktop in pure Rust)...");
        println!("Working dir: {:?}", self.working_dir);
        println!("Window geometry: {}x{}", self.layout.window_rect.width, self.layout.window_rect.height);
        println!("Account status: {:?}", self.account.state());
        0
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let launcher = Launcher::new(args);
    std::process::exit(launcher.run());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_default_args() {
        let launcher = Launcher::new(vec!["omnigram".to_string()]);
        assert_eq!(launcher.arguments().len(), 1);
        assert_eq!(launcher.layout().window_rect.width, 800);
        assert_eq!(launcher.account().index(), 0);
    }

    #[test]
    fn test_launcher_custom_workdir() {
        let launcher = Launcher::new(vec![
            "omnigram".to_string(),
            "-workdir".to_string(),
            "/tmp/custom_tdesktop".to_string(),
        ]);
        assert_eq!(launcher.working_dir(), Path::new("/tmp/custom_tdesktop"));
    }
}
