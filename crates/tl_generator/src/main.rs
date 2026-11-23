//! CLI: `tl_generator <входы.tl...> -o <выход.rs>`
//!
//! Проверяет id всех конструкторов (оракул CRC-32) и генерирует Rust.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut inputs = Vec::new();
    let mut output = None;
    let mut next_out = false;
    for arg in &args {
        if next_out {
            output = Some(arg.clone());
            next_out = false;
        } else if arg == "-o" {
            next_out = true;
        } else if let Some(rest) = arg.strip_prefix("-o") {
            output = Some(rest.to_string());
        } else {
            inputs.push(arg.clone());
        }
    }
    if inputs.is_empty() || output.is_none() {
        eprintln!("Использование: tl_generator <входы.tl...> -o <выход.rs>");
        return ExitCode::FAILURE;
    }

    let refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
    let scheme = match tl_generator::parse_files(&refs) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Ошибка разбора: {}", e.0);
            return ExitCode::FAILURE;
        }
    };

    let code = tl_generator::generate(&scheme, "Схема MTProto.");
    if let Err(e) = std::fs::write(output.unwrap(), code) {
        eprintln!("Ошибка записи: {e}");
        return ExitCode::FAILURE;
    }
    eprintln!("ok: {} конструкторов, LAYER {}", scheme.constructors.len(), scheme.layer);
    ExitCode::SUCCESS
}
