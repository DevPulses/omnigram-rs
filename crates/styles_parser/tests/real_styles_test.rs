use std::path::PathBuf;
use styles_parser::{Palette, StyleSheet, Value};

#[test]
fn test_parse_real_colors_palette() {
    let path = PathBuf::from("tdesktop/Telegram/lib_ui/ui/colors.palette");
    if !path.exists() {
        eprintln!("Skipping: colors.palette not found");
        return;
    }
    let content = std::fs::read_to_string(&path).expect("Read colors.palette");
    let palette = Palette::parse(&content).expect("Parse colors.palette");

    assert!(palette.colors.len() > 100, "Expected > 100 colors, got {}", palette.colors.len());
    let window_bg = palette.get("windowBg").expect("windowBg present");
    assert_eq!((window_bg.r, window_bg.g, window_bg.b, window_bg.a), (255, 255, 255, 255));

    let window_fg = palette.get("windowFg").expect("windowFg present");
    assert_eq!((window_fg.r, window_fg.g, window_fg.b, window_fg.a), (0, 0, 0, 255));
}

#[test]
fn test_parse_real_basic_style() {
    let path = PathBuf::from("tdesktop/Telegram/lib_ui/ui/basic.style");
    if !path.exists() {
        eprintln!("Skipping: basic.style not found");
        return;
    }

    let search_paths = vec![
        PathBuf::from("tdesktop/Telegram/lib_ui"),
        PathBuf::from("tdesktop/Telegram/SourceFiles"),
    ];

    let mut sheet = StyleSheet::new();
    sheet.load_file(&path, &search_paths).expect("Load basic.style");

    assert!(sheet.all_struct_defs.contains_key("TextStyle"));
    assert!(sheet.all_struct_defs.contains_key("QuoteStyle"));
    assert_eq!(sheet.get_variable("kLinkUnderlineNever"), Some(&Value::Int(0)));
    assert_eq!(sheet.get_variable("kLinkUnderlineActive"), Some(&Value::Int(1)));
}

#[test]
fn test_parse_real_window_style() {
    let path = PathBuf::from("tdesktop/Telegram/SourceFiles/window/window.style");
    if !path.exists() {
        eprintln!("Skipping: window.style not found");
        return;
    }

    let search_paths = vec![
        PathBuf::from("tdesktop/Telegram/SourceFiles"),
        PathBuf::from("tdesktop/Telegram/lib_ui"),
    ];

    let mut sheet = StyleSheet::new();
    let res = sheet.load_file(&path, &search_paths);
    match res {
        Ok(_) => {
            assert_eq!(sheet.get_pixels("windowMinWidth"), Some(380));
            assert_eq!(sheet.get_pixels("windowMinHeight"), Some(480));
            assert_eq!(sheet.get_pixels("windowDefaultWidth"), Some(800));
            assert_eq!(sheet.get_pixels("windowDefaultHeight"), Some(600));
        }
        Err(e) => {
            panic!("Failed to load window.style: {}", e);
        }
    }
}
