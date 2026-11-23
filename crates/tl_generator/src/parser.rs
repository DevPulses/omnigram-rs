//! Парсер `.tl`-схемы.
//!
//! Порт `readInputs` + цикла разбора `readAndGenerate` из
//! `lib_tl/tl/generate_tl.py`: секции `---types---`/`---functions---`,
//! комментарии (включая `// LAYER N`), пропуски, вычисление и проверка
//! id конструкторов через CRC-32 «чистой» строки.

use crate::crc32::crc32;

/// Конструктор TL (`name#id params = Result;`).
#[derive(Debug, Clone)]
pub struct Constructor {
    /// Имя без namespace-точек, как в схеме (например `resPQ`).
    pub name: String,
    /// Имя с точками в оригинальном виде (`messages.dialogs`).
    pub original_name: String,
    /// Вычисленный из cleanline CRC-32 (или явный, если совпадает).
    pub id: u32,
    /// Явно указанный в схеме id, если есть.
    pub explicit_id: Option<u32>,
    /// true — это определение из секции `---functions---`.
    pub is_function: bool,
    /// Параметры; шаблонные `{X:Type}` сюда не попадают.
    pub params: Vec<Param>,
    /// Имя шаблонного параметра (`X`), если конструктор шаблонный.
    pub template: Option<String>,
    /// Тип результата (строкой из схемы, включая `Vector<...>`).
    pub result: String,
}

/// Параметр конструктора.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    /// Сырой тип из схемы: `int`, `flags`, `flags.0?InputFile`,
    /// `Vector<long>`, `!X`, `true` и т.п.
    pub ty: String,
}

impl Param {
    /// Условный параметр (`name.flags.N?Type`) → (имя поля флагов, бит, тип).
    pub fn condition(&self) -> Option<(&str, u32, &str)> {
        let (head, tail) = self.ty.split_once('?')?;
        let (flags, bit) = head.split_once('.')?;
        Some((flags, bit.parse().ok()?, tail))
    }

    /// Параметр-флаг-контейнер (`flags:#`, `flags2:#`).
    pub fn is_flags(&self) -> bool {
        self.ty == "#"
    }

    /// Ссылка на шаблонный тип (`!X`).
    pub fn is_template_ref(&self) -> bool {
        self.ty.starts_with('!')
    }

    /// Тривиальное условие `flags.N?true`.
    pub fn is_trivial_condition(&self) -> bool {
        matches!(self.condition(), Some((_, _, "true")))
    }
}

/// Разобранная схема.
#[derive(Debug, Default)]
pub struct Scheme {
    pub layer: u32,
    pub constructors: Vec<Constructor>,
}

/// Строки встроенных определений, вырезаемые как в `codegen_scheme.py`
/// (`skip`): они выражаются нативными типами генератора.
pub const SKIP_LINES: &[&str] = &[
    "int ? = Int;",
    "long ? = Long;",
    "double ? = Double;",
    "string ? = String;",
    "vector {t:Type} # [ t ] = Vector t;",
    "int128 4*[ int ] = Int128;",
    "int256 8*[ int ] = Int256;",
    "vector#1cb5c415 {t:Type} # [ t ] = Vector t;",
];

/// Исключения: явный id не совпадает с вычисленным, но конструктор
/// остаётся (`typeIdExceptions` из `codegen_scheme.py`).
pub const TYPE_ID_EXCEPTIONS: &[&str] = &[
    "channel#c88974ac",
    "ipPortSecret#37982646",
    "accessPointRule#4679b65f",
    "help.configSimple#5a592a6c",
    "messageReplies#81834865",
];

/// Ошибка разбора схемы.
#[derive(Debug)]
pub struct ParseError(pub String);

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// `cleanline` — строка, от CRC которой считается id конструктора
/// (точный порт из `generate_tl.py`).
pub fn cleanline(name: &str, params_raw: &str, result: &str) -> String {
    let mut s = format!("{name}{params_raw}= {result}");

    // Убрать условные true-параметры: ' name:flags2?.N?true'.
    s = remove_flag_true(&s);

    // '<' и '>' → пробел, затем ОДНОпроходное схлопывание двойных
    // пробелов (как str.replace('  ', ' ') в Python — не рекурсивно).
    s = s.replace('<', " ").replace('>', " ");
    s = s.replace("  ", " ");

    // strip() по краям.
    s.trim().to_string()
}

/// Эквивалент `re.sub(r' [a-zA-Z0-9_]+\:flags2?\.[0-9]+\?true', '', s)`.
fn remove_flag_true(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b' ' {
            // Попытка сматчить шаблон начиная с этого пробела.
            let rest = &s[i + 1..];
            if let Some(end) = match_flag_true(rest) {
                i += 1 + end;
                continue;
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

/// Длина совпадения `name:flags.N?true` / `name:flags2.N?true` в начале.
fn match_flag_true(s: &str) -> Option<usize> {
    // Токен ограничен пробелом (regex матчит ровно один токен).
    let tok_end = s.find(' ').unwrap_or(s.len());
    let tok = &s[..tok_end];
    let (name, rest) = tok.split_once(':')?;
    if name.is_empty() || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return None;
    }
    let flags = rest.strip_prefix("flags")?;
    let flags = flags.strip_prefix('2').unwrap_or(flags);
    let bit = flags.strip_prefix('.')?;
    let (bit, tail) = bit.split_once('?')?;
    if bit.is_empty() || !bit.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if tail != "true" {
        return None;
    }
    Some(tok_end)
}

/// Эквивалент `binascii.a2b_qp` для ASCII-строк без переводов строк:
/// `=XX` (две hex-цифры) → байт, иначе `=` остаётся как есть.
fn a2b_qp(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'=' && i + 2 < b.len() + 1 && i + 2 < b.len() + 1 {
            let hex = b.get(i + 1..i + 3);
            if let Some(h) = hex {
                if let (Some(hi), Some(lo)) = (hex_val(h[0]), hex_val(h[1])) {
                    out.push(hi << 4 | lo);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Вычислить id конструктора из cleanline (CRC-32).
pub fn constructor_id(name: &str, params_raw: &str, result: &str) -> u32 {
    crc32(a2b_qp(&cleanline(name, params_raw, result)).as_slice())
}

/// Разобрать `.tl`-файлы (эквивалент `readInputs`: все входы склеиваются,
/// каждый начинается с `---types---`).
pub fn parse_files(inputs: &[&str]) -> Result<Scheme, ParseError> {
    let mut scheme = Scheme::default();
    let mut accumulated_comments = String::new();

    for (file_index, input) in inputs.iter().enumerate() {
        let text = std::fs::read_to_string(input)
            .map_err(|e| ParseError(format!("{input}: {e}")))?;
        // Как readInputs: каждый входной файл начинается с
        // '---types---' (python дописывает маркер перед содержимым).
        let mut is_function = false;

        for line in text.lines() {
            let (line, comment) = match line.find("//") {
                Some(pos) => (&line[..pos], line[pos + 2..].to_string()),
                None => (line, String::new()),
            };

            // LAYER-комментарий (как `// LAYER 233`): python ищет его
            // в сырой строке до вырезания комментария.
            let layerline = line.trim_start().strip_prefix("LAYER ");
            if let Some(rest) = comment.trim_start().strip_prefix("LAYER ") {
                if let Ok(layer) = rest.trim().parse() {
                    scheme.layer = layer;
                }
                continue;
            }
            let _ = layerline;

            if line.trim() == "---functions---" {
                is_function = true;
                continue;
            }
            if line.trim() == "---types---" {
                is_function = false;
                continue;
            }

            if is_blank(line) {
                if comment.is_empty() {
                    accumulated_comments.clear();
                } else {
                    accumulated_comments.push(' ');
                    accumulated_comments.push_str(&comment);
                }
                continue;
            }

            if SKIP_LINES.contains(&line.trim()) {
                continue;
            }

            if let Some(def) = parse_definition(line, is_function, &scheme)? {
                scheme.constructors.push(def);
            }
            accumulated_comments.clear();
        }
    }
    Ok(scheme)
}

/// Разобрать одно определение `name[#id] params = Result;`.
/// `Ok(None)` — конструктор пропущен (mismatch id, как в generate_tl.py).
fn parse_definition(line: &str, is_function: bool, scheme: &Scheme) -> Result<Option<Constructor>, ParseError> {
    let bad = || ParseError(format!("Bad line found: {line}"));

    let body = line.trim().strip_suffix(';').ok_or_else(bad)?;
    let (head, result) = body.split_once('=').ok_or_else(bad)?;
    let result = result.trim();

    // Имя и явный id.
    let head_trim = head.trim_start();
    let (name_full, params_raw) = match head_trim.find(char::is_whitespace) {
        Some(pos) => head_trim.split_at(pos),
        None => (head_trim, ""),
    };
    let (name, explicit_id) = match name_full.split_once('#') {
        Some((n, hex_id)) => {
            let id = u32::from_str_radix(hex_id, 16).map_err(|_| bad())?;
            (n.to_string(), Some(id))
        }
        None => (name_full.to_string(), None),
    };
    if name.is_empty() {
        return Err(bad());
    }

    // Параметры.
    let mut params = Vec::new();
    let mut template = None;
    for token in params_raw.split_whitespace() {
        if let Some(inner) = token.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
            // {X:Type}
            let tname = inner.split_once(':').map(|(n, _)| n).unwrap_or(inner);
            template = Some(tname.to_string());
            continue;
        }
        let (pname, pty) = token.split_once(':').ok_or_else(bad)?;
        params.push(Param {
            name: pname.to_string(),
            ty: pty.to_string(),
        });
    }

    // Вычислить и сверить id.
    let mut id = constructor_id(&name, params_raw, result);
    if let Some(explicit) = explicit_id {
        // Ведущие нули в схеме не значимы (python: while typeid[0]=='0').
        if id != explicit {
            let key = format!("{name}#{explicit:08x}");
            if !TYPE_ID_EXCEPTIONS.contains(&key.as_str()) {
                // В оригинале (generate_tl.py) — warning и пропуск
                // конструктора, но это резало бы канонические
                // конструкторы TL, чей id вычислен от определения на
                // момент создания (поля менялись, id — нет; tdesktop
                // эти конструкторы использует). Поэтому: предупреждение,
                // конструктор остаётся с явным (каноническим) id.
                let cl = cleanline(&name, params_raw, result);
                eprintln!(
                    "Warning: counted {id:#x} mismatch with provided {explicit:#x} ({key}, {cl})"
                );
            }
            id = explicit;
        }
    }

    // Ведущие нули: явный id вида #05162463 значим как 0x05162463.
    if explicit_id.is_none() {
        let _ = id;
    }

    let _ = scheme;
    Ok(Some(Constructor {
        original_name: name_full_no_hash(name_full, explicit_id.is_some()),
        name,
        id,
        explicit_id,
        is_function,
        params,
        template,
        result: result.to_string(),
    }))
}

/// `original_name` — имя вместе с namespace-точками (у нас name уже без
/// правок, точки сохраняются; отдельная функция для читаемости).
fn name_full_no_hash(name_full: &str, _had_id: bool) -> String {
    name_full.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanline_matches_python() {
        // Проверено против generate_tl.py + binascii.crc32.
        // Как в реальном потоке: за последним параметром идёт пробел
        // перед '='.
        let cl = cleanline("req_pq", " nonce:int128 ", "ResPQ");
        assert_eq!(cl, "req_pq nonce:int128 = ResPQ");
        assert_eq!(constructor_id("req_pq", " nonce:int128 ", "ResPQ"), 0x6046_9778);

        let cl = cleanline("destroy_auth_key", " ", "DestroyAuthKeyRes");
        assert_eq!(cl, "destroy_auth_key = DestroyAuthKeyRes");
        assert_eq!(
            constructor_id("destroy_auth_key", " ", "DestroyAuthKeyRes"),
            0xd143_5160
        );

        // resPQ#05162463 — ведущий ноль в явном id.
        let id = constructor_id(
            "resPQ",
            " nonce:int128 server_nonce:int128 pq:string server_public_key_fingerprints:Vector<long>",
            "ResPQ",
        );
        assert_eq!(id, 0x0516_2463);
    }

    #[test]
    fn cleanline_strips_flag_true_and_angles() {
        // inputMediaUploadedPhoto без flags-true условий.
        let cl = cleanline(
            "inputMediaUploadedPhoto",
            " flags:# spoiler:flags.2?true live_photo:flags.3?true file:InputFile stickers:flags.0?Vector<InputDocument> ttl_seconds:flags.1?int video:flags.3?InputDocument ",
            "InputMedia",
        );
        assert_eq!(
            cl,
            "inputMediaUploadedPhoto flags:# file:InputFile stickers:flags.0?Vector InputDocument ttl_seconds:flags.1?int video:flags.3?InputDocument = InputMedia"
        );
    }

    #[test]
    fn qp_decode_identity() {
        assert_eq!(a2b_qp("ab= cd"), b"ab= cd");
        assert_eq!(a2b_qp("a=41b"), b"aAb");
    }
}


