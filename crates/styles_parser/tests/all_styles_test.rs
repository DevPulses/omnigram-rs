use std::path::{Path, PathBuf};
use styles_parser::StyleSheet;

#[test]
fn test_parse_all_real_style_files() {
    let base_dir = PathBuf::from("tdesktop/Telegram");
    if !base_dir.exists() {
        eprintln!("Skipping: tdesktop/Telegram not found");
        return;
    }

    let search_paths = vec![
        PathBuf::from("tdesktop/Telegram/SourceFiles"),
        PathBuf::from("tdesktop/Telegram/lib_ui"),
        PathBuf::from("tdesktop/Telegram/Resources"),
    ];

    let mut style_files = Vec::new();
    collect_style_files(&base_dir, &mut style_files);

    assert!(!style_files.is_empty(), "Found no style files");

    let mut succeeded = 0;
    let mut failed = Vec::new();

    for file in &style_files {
        let mut sheet = StyleSheet::new();
        match sheet.load_file(file, &search_paths) {
            Ok(_) => succeeded += 1,
            Err(e) => failed.push((file.clone(), e)),
        }
    }

    println!("Parsed {} style files successfully, {} failed", succeeded, failed.len());
    if !failed.is_empty() {
        for (f, err) in &failed {
            eprintln!("Failed {:?}: {}", f, err);
        }
        panic!("Failed parsing {} files out of {}", failed.len(), style_files.len());
    }
}

fn collect_style_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_style_files(&path, out);
            } else if let Some(ext) = path.extension() {
                if ext == "style" {
                    out.push(path);
                }
            }
        }
    }
}
