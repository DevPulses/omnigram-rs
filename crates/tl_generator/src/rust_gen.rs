//! Генерация Rust-кода из разобранной схемы.
//!
//! Семантика соответствует `read-write`-секции `generate_tl.py`:
//! - у каждого конструктора — константа id, чтение/запись полей;
//! - `flags:#`/`flags2:#` — u32-поля, читаются/пишутся всегда;
//! - условные поля `name.flags.N?T` — `Option<T>`, только при бите;
//! - `flags.N?true` — поля не создаёт, состояние живёт во флагах;
//! - шаблонные конструкторы (`{X:Type}`, параметр `!X`) — дженерики
//!   с трейтами `TlBoxedRead`/`TlBoxedWrite`;
//! - capitalized типы (`Bool`, `InputFile`) — boxed (с конструктором),
//!   строчные (`int`, `long`, `Vector<long>`) — bare.

use crate::parser::{Constructor, Param, Scheme};
use std::fmt::Write;

/// Контекст генерации: схема для разрешения типов.
struct GenCtx<'a> {
    scheme: &'a Scheme,
    /// Мемоизация: rust-имя типа → типы, которых он достигает транзитивно.
    reach: std::cell::RefCell<HashMap<String, HashSet<String>>>,
}

use std::collections::{HashMap, HashSet};

impl<'a> GenCtx<'a> {
    /// Прямые ссылки типа `ty` (rust-имя) на другие определённые типы.
    fn direct_refs(&self, ty: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for c in &self.scheme.constructors {
            if c.is_function {
                continue;
            }
            if rust_type_name(&c.result) != ty {
                continue;
            }
            for p in &c.params {
                self.collect_refs(p, &mut out);
            }
        }
        out
    }

    fn collect_refs(&self, p: &Param, out: &mut HashSet<String>) {
        let raw = p.ty.as_str();
        if let Some((_, _, inner)) = p.condition() {
            self.collect_refs(
                &Param {
                    name: String::new(),
                    ty: inner.to_string(),
                },
                out,
            );
            return;
        }
        if let Some(v) = vector_inner(raw) {
            self.collect_refs(
                &Param {
                    name: String::new(),
                    ty: v.to_string(),
                },
                out,
            );
            return;
        }
        if p.is_flags() || p.is_template_ref() || builtin_bare(raw).is_some() {
            return;
        }
        if self.known_type(raw) {
            out.insert(rust_type_name(raw));
        }
    }

    /// Достигает ли тип `from` тип `to` транзитивно (from != to).
    fn reaches(&self, from: &str, to: &str) -> bool {
        {
            let memo = self.reach.borrow();
            if let Some(set) = memo.get(from) {
                return set.contains(to);
            }
        }
        // DFS без мемо на текущем пути.
        let mut seen: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = self.direct_refs(from).into_iter().collect();
        while let Some(t) = stack.pop() {
            if t == to {
                return true;
            }
            if !seen.insert(t.clone()) {
                continue;
            }
            for n in self.direct_refs(&t) {
                stack.push(n);
            }
        }
        false
    }
}

/// Сгенерировать содержимое одного .rs-файла.
pub fn generate(scheme: &Scheme, module_doc: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Сгенерировано tl_generator из .tl-схемы (LAYER {}).",
        scheme.layer
    );
    let _ = writeln!(out, "// {module_doc}");
    let _ = writeln!(out, "// НЕ редактировать вручную.");
    let _ = writeln!(out, "#![allow(clippy::all, unused_variables, dead_code)]");
    out.push('\n');
    out.push_str(PRELUDE);
    out.push('\n');

    let _ = writeln!(out, "/// Конструкторы (аналог mtpc_*).");
    for c in &scheme.constructors {
        let _ = writeln!(out, "pub const {}: u32 = {:#010x};", screaming(&c.name), c.id);
    }
    out.push('\n');

    let ctx = GenCtx {
        scheme,
        reach: std::cell::RefCell::new(HashMap::new()),
    };
    for (type_name, cons) in ctx.group_types() {
        gen_enum(&mut out, &ctx, &type_name, &cons);
    }

    let funcs: Vec<&Constructor> = scheme.constructors.iter().filter(|c| c.is_function).collect();
    if !funcs.is_empty() {
        out.push_str("/// RPC-методы (запросы).\npub mod functions {\n    use super::*;\n\n");
        for f in funcs {
            gen_function(&mut out, &ctx, f);
        }
        out.push_str("}\n");
    }

    out
}

impl<'a> GenCtx<'a> {
    /// Группировка конструкторов типов по результату (порядок сохранён).
    fn group_types(&self) -> Vec<(String, Vec<&Constructor>)> {
        let mut groups: Vec<(String, Vec<&Constructor>)> = Vec::new();
        for c in &self.scheme.constructors {
            if c.is_function {
                continue;
            }
            let t = rust_type_name(&c.result);
            match groups.iter_mut().find(|(n, _)| *n == t) {
                Some(g) => g.1.push(c),
                None => groups.push((t, vec![c])),
            }
        }
        groups
    }

    /// Известен ли тип (определён в схеме).
    fn known_type(&self, ty: &str) -> bool {
        let want = rust_type_name(ty);
        self.scheme
            .constructors
            .iter()
            .filter(|c| !c.is_function)
            .any(|c| rust_type_name(&c.result) == want)
    }

    /// Boxed ли тип: capitalized (`Bool`, `InputFile`). Строчные ссылки
    /// (`auth.SentCode`) — bare, допустимы только для одного конструктора.
    fn is_boxed(&self, ty: &str) -> bool {
        if ty == "#" || builtin_bare(ty).is_some() {
            return false;
        }
        // namespaced-ссылки (`auth.SentCodeType`) сериализуются boxed:
        // мульти-конструкторный union иначе не передать.
        if ty.contains('.') {
            return true;
        }
        ty.chars().next().unwrap_or('a').is_uppercase()
    }

    /// bare-ссылка на определённый тип корректна только при одном
    /// конструкторе (как в generate_tl.py: "Complex bare type found").
    fn check_bare_single(&self, ty: &str) {
        if self.is_boxed(ty) || builtin_bare(ty).is_some() || ty == "#" {
            return;
        }
        if !self.known_type(ty) {
            panic!("Неизвестный тип параметра: {ty}");
        }
        let want = rust_type_name(ty);
        let count = self
            .scheme
            .constructors
            .iter()
            .filter(|c| !c.is_function && rust_type_name(&c.result) == want)
            .count();
        assert!(count == 1, "Complex bare type found: {ty} ({count} конструкторов)");
    }
}

/// Типы, выражаемые нативно (bare). Возвращает растовый тип.
fn builtin_bare(ty: &str) -> Option<&'static str> {
    Some(match ty {
        "int" => "i32",
        "long" => "u64",
        "double" => "f64",
        "string" | "bytes" => "Vec<u8>",
        "int128" => "Int128",
        "int256" => "Int256",
        _ => return None,
    })
}

impl<'a> GenCtx<'a> {
    /// Растовый тип параметра; `enclosing` — имя типа-владельца для
    /// определения рекурсии (циклические поля оборачиваются в Box).
    fn field_type(&self, p: &Param, enclosing: &str) -> String {
        let raw = p.ty.as_str();
        if p.is_flags() {
            return "u32".into();
        }
        if p.is_template_ref() {
            return "T".into();
        }
        if let Some((_, _, inner)) = p.condition() {
            let inner_param = Param {
                name: p.name.clone(),
                ty: inner.to_string(),
            };
            return format!("Option<{}>", self.field_type(&inner_param, enclosing));
        }
        if let Some(v) = vector_inner(raw) {
            let elem = Param {
                name: String::new(),
                ty: v.to_string(),
            };
            return format!("Vec<{}>", self.field_type(&elem, enclosing));
        }
        if let Some(rust) = builtin_bare(raw) {
            return rust.into();
        }
        let ty = rust_type_name(raw);
        if !enclosing.is_empty() && self.reaches(&ty, enclosing) {
            format!("Box<{ty}>")
        } else {
            ty
        }
    }

    /// Выражение чтения значения типа `p.ty` (Option-обёртку ставит
    /// вызывающий код для условных полей).
    fn value_read(&self, p: &Param, enclosing: &str) -> String {
        format!("{}?", self.value_read_result(p, enclosing))
    }

    /// Выражение чтения, дающее `Result<T, Eof>`, без завершающего `?`
    /// (для замыканий векторов и условных обёрток Some).
    fn value_read_result(&self, p: &Param, enclosing: &str) -> String {
        // Условное поле `name.flags.N?T`: читается как T.
        if let Some((_, _, inner)) = p.condition() {
            return self.value_read_result(
                &Param {
                    name: p.name.clone(),
                    ty: inner.to_string(),
                },
                enclosing,
            );
        }
        let raw = p.ty.as_str();
        if p.is_flags() {
            return "r.get()".into();
        }
        if p.is_template_ref() {
            return "{ let cons = r.get()?; T::tl_read_boxed(r, cons) }".into();
        }
        if let Some(v) = vector_inner(raw) {
            let elem = Param {
                name: String::new(),
                ty: v.to_string(),
            };
            // Примитивные элементы — bare (Vector<long> = счётчик + long'и);
            // определённые типы — boxed, со своим конструктором.
            if builtin_bare(v).is_some() {
                let body = self.value_read_result(&elem, enclosing);
                return format!("read_vector(r, |r| {body})");
            }
            let et = self.field_type(&elem, enclosing);
            return format!(
                "read_vector(r, |r| {{ let cons = r.get()?; <{et} as TlBoxedRead>::tl_read_boxed(r, cons) }})"
            );
        }
        if let Some(rust) = builtin_bare(raw) {
            return match rust {
                "i32" => "read_int(r)".into(),
                "u64" => "read_long(r)".into(),
                "f64" => "read_double(r)".into(),
                "Vec<u8>" => "read_string(r)".into(),
                "Int128" => "read_int128(r)".into(),
                "Int256" => "read_int256(r)".into(),
                _ => unreachable!(),
            };
        }
        // Определённые типы всегда boxed (как MTP-классы в C++-кодогене
        // tdesktop: даже bare-ссылки читают/пишут конструктор).
        let ft = self.field_type(p, enclosing);
        format!("{{ let cons = r.get()?; <{ft} as TlBoxedRead>::tl_read_boxed(r, cons) }}")
    }

    /// Выражение записи значения (`x` — имя переменной).
    fn value_write(&self, p: &Param, x: &str, enclosing: &str, by_ref: bool) -> String {
        // Условное поле: пишется как T (обёртку Some снимает вызывающий).
        if let Some((_, _, inner)) = p.condition() {
            return self.value_write(
                &Param {
                    name: p.name.clone(),
                    ty: inner.to_string(),
                },
                x,
                enclosing,
                by_ref,
            );
        }
        let raw = p.ty.as_str();
        if p.is_flags() {
            let v = if by_ref { format!("*{x}") } else { x.to_string() };
            return format!("w.put({v});");
        }
        if p.is_template_ref() {
            return format!("{x}.tl_write_boxed(w);");
        }
        if let Some(v) = vector_inner(raw) {
            // Вектор в схеме всегда bare: счётчик + элементы; элементы
            // boxed при capitalized-типе (Vector<Chat>), иначе bare.
            let elem = Param {
                name: String::new(),
                ty: v.to_string(),
            };
            let item = if builtin_bare(v).is_some() {
                self.value_write(&elem, "_elem", enclosing, true)
            } else {
                "_elem.tl_write_boxed(w);".to_string()
            };
            return format!(
                "w.put({x}.len() as u32);\n                for _elem in {x}.iter() {{ {item} }}"
            );
        }
        if let Some(rust) = builtin_bare(raw) {
            let v = if by_ref { format!("*{x}") } else { x.to_string() };
            return match rust {
                "i32" => format!("write_int(w, {v});"),
                "u64" => format!("write_long(w, {v});"),
                "f64" => format!("write_double(w, {v});"),
                "Vec<u8>" => format!("write_string(w, {x});"),
                "Int128" => format!("write_int128(w, {v});"),
                "Int256" => format!("write_int256(w, {v});"),
                _ => unreachable!(),
            };
        }
        format!("{x}.tl_write_boxed(w);")
    }
}

/// Внутренний тип `Vector<...>`/`vector<...>` (boxед и bare векторы).
fn vector_inner(raw: &str) -> Option<&str> {
    raw.strip_prefix("Vector<")
        .or_else(|| raw.strip_prefix("vector<"))
        .and_then(|s| s.strip_suffix('>'))
}

/// Список сериализуемых полей (без тривиальных `?true`).
fn real_fields(c: &Constructor) -> Vec<&Param> {
    c.params.iter().filter(|p| !p.is_trivial_condition()).collect()
}

// ---------------- генерация enum'ов ----------------

fn gen_enum(out: &mut String, ctx: &GenCtx, type_name: &str, cons: &[&Constructor]) {
    out.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    let _ = writeln!(out, "pub enum {type_name} {{");
    for c in cons {
        let _ = writeln!(out, "    {},", variant_signature(ctx, c, type_name));
    }
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl {type_name} {{");
    for c in cons {
        let _ = writeln!(out, "    pub const {}: u32 = {:#010x};", screaming(&c.name), c.id);
    }

    out.push_str("\n    /// Чтение тела по уже известному конструктору.\n");
    out.push_str("    pub fn read_unboxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {\n");
    out.push_str("        match cons {\n");
    for c in cons {
        let _ = writeln!(
            out,
            "            Self::{} => {{",
            screaming(&c.name),
        );
        // Читаем все поля последовательно в порядке параметров (флаги
        // могут стоять не первыми, например wallPaper). Условные поля
        // ссылаются на уже прочитанные let-локали.
        let mut field_locals: Vec<String> = Vec::new();
        let mut reads = String::new();
        for p in real_fields(c) {
            let f = field_name(&p.name);
            if let Some((flags, bit, _)) = p.condition() {
                let expr = ctx.value_read(p, type_name);
                let _ = writeln!(
                    reads,
                    "            let {f} = if {flags} & (1u32 << {bit}) != 0 {{ Some({expr}) }} else {{ None }};"
                );
            } else {
                let expr = ctx.value_read(p, type_name);
                let _ = writeln!(reads, "            let {f} = {expr};");
            }
            field_locals.push(f);
        }
        out.push_str(&reads);
        let _ = writeln!(
            out,
            "            Ok(Self::{} {{",
            pascal_case(&c.name)
        );
        for f in &field_locals {
            let _ = writeln!(out, "                {f},");
        }
        out.push_str("            })\n");
        out.push_str("        }\n");
    }
    out.push_str("            _ => Err(Eof),\n");
    out.push_str("        }\n    }\n");

    out.push_str("\n    /// Boxed-чтение: конструктор читается из потока.\n");
    out.push_str("    pub fn read_boxed(r: &mut Reader) -> Result<Self, Eof> {\n");
    out.push_str("        let cons = r.get()?;\n        Self::read_unboxed(r, cons)\n    }\n");

    out.push_str("\n    /// Запись тела без конструктора (bare).\n");
    out.push_str("    pub fn write_unboxed(&self, w: &mut Writer) {\n");
    out.push_str("        match self {\n");
    for c in cons {
        // Биндинги f0, f1... — поле `w` затенило бы писателя.
        let binds: Vec<String> = real_fields(c)
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}: f{}", field_name(&p.name), i))
            .collect();
        if binds.is_empty() {
            let _ = writeln!(out, "            Self::{} => {{", pascal_case(&c.name));
        } else {
            let _ = writeln!(
                out,
                "            Self::{} {{ {} }} => {{",
                pascal_case(&c.name),
                binds.join(", ")
            );
        }
        // Имя flags-параметра → биндинг f<i> (в паттерне поле переименован).
        let binds_by_name: HashMap<&str, String> = real_fields(c)
            .iter()
            .enumerate()
            .map(|(i, p)| (p.name.as_str(), format!("f{i}")))
            .collect();
        for (i, p) in real_fields(c).iter().enumerate() {
            let v = format!("f{i}");
            if let Some((flags, bit, _)) = p.condition() {
                let fbind = binds_by_name.get(flags).cloned().unwrap_or_else(|| flags.to_string());
                let _ = writeln!(out, "                if {fbind} & (1u32 << {bit}) != 0 {{");
                let _ = writeln!(out, "                    if let Some({v}) = {v} {{");
                let _ = writeln!(
                    out,
                    "                        {}",
                    ctx.value_write(p, &v, type_name, true)
                );
                out.push_str("                    }\n");
                out.push_str("                }\n");
            } else {
                let _ = writeln!(
                    out,
                    "                {}",
                    ctx.value_write(p, &v, type_name, true)
                );
            }
        }
        out.push_str("            }\n");
    }
    out.push_str("        }\n    }\n");

    out.push_str("\n    /// Boxed-запись: конструктор, затем тело.\n");
    out.push_str("    pub fn write_boxed(&self, w: &mut Writer) {\n");
    out.push_str("        match self {\n");
    for c in cons {
        let _ = writeln!(
            out,
            "            Self::{} {{ .. }} => {{ w.put(Self::{}); self.write_unboxed(w); }},",
            pascal_case(&c.name),
            screaming(&c.name)
        );
    }
    out.push_str("        }\n    }\n");
    out.push_str("}\n\n");

    // Адаптеры трейтов. Boxed-использование всегда через TlBoxedRead/
    // TlBoxedWrite; bare (ссылки вида `auth.SentCode`, элементы
    // `Vector<future_salt>`) — тело конструктора без id, допустимо
    // только для single-constructor типов (проверяет check_bare_single).
    let _ = writeln!(out, "impl TlBoxedRead for {type_name} {{");
    out.push_str("    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {\n");
    out.push_str("        Self::read_unboxed(r, cons)\n    }\n}\n");
    let _ = writeln!(out, "impl TlBoxedWrite for {type_name} {{");
    out.push_str("    fn tl_write_boxed(&self, w: &mut Writer) {\n        self.write_boxed(w)\n    }\n}\n");
    let _ = writeln!(out, "impl TlRead for {type_name} {{");
    if cons.len() == 1 {
        let id = screaming(&cons[0].name);
        out.push_str("    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {\n");
        out.push_str(format!("        Self::read_unboxed(r, Self::{id})\n    }}\n}}\n").as_str());
    } else {
        out.push_str("    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {\n");
        out.push_str("        let cons = r.get()?;\n        Self::read_unboxed(r, cons)\n    }\n}\n");
    }
    let _ = writeln!(out, "impl TlWrite for {type_name} {{");
    out.push_str("    fn tl_write(&self, w: &mut Writer) {\n        self.write_unboxed(w)\n    }\n}\n\n");
}

/// Сигнатура варианта перечисления с именованными полями.
fn variant_signature(ctx: &GenCtx, c: &Constructor, type_name: &str) -> String {
    let mut s = pascal_case(&c.name);
    let fields = real_fields(c);
    if !fields.is_empty() {
        s.push_str(" { ");
        for p in &fields {
            let _ = write!(s, "{}: {}, ", field_name(&p.name), ctx.field_type(p, type_name));
        }
        s.push('}');
    }
    s
}

// ---------------- функции (запросы) ----------------

fn gen_function(out: &mut String, ctx: &GenCtx, f: &Constructor) {
    let type_name = "";
    let struct_name = pascal_case(&f.name);
    let generic = f.template.as_ref().map(|_| "<T>");
    let generic_def = f.template.as_ref().map(|_| "<T>").unwrap_or("");

    if f.template.is_some() {
        let _ = writeln!(out, "#[derive(Debug, Clone, PartialEq)]");
        let _ = writeln!(out, "pub struct {struct_name}{generic_def} {{");
    } else {
        let _ = writeln!(out, "#[derive(Debug, Clone, PartialEq)]");
        let _ = writeln!(out, "pub struct {struct_name} {{");
    }
    for p in real_fields(f) {
        let _ = writeln!(out, "    pub {}: {},", field_name(&p.name), ctx.field_type(p, type_name));
    }
    out.push_str("}\n\n");

    let self_ty = if f.template.is_some() {
        format!("{struct_name}<T>")
    } else {
        struct_name.clone()
    };
    if f.template.is_some() {
        let _ = writeln!(out, "impl<T: TlBoxedWrite> {self_ty} {{");
    } else {
        let _ = writeln!(out, "impl {self_ty} {{");
    }
    let _ = writeln!(out, "    pub const ID: u32 = {:#010x};", f.id);
    out.push_str("\n    /// Boxed-запрос: конструктор, затем поля.\n");
    out.push_str("    pub fn write_boxed(&self, w: &mut Writer) {\n");
    out.push_str("        w.put(Self::ID);\n");
    for p in real_fields(f) {
        let fname = field_name(&p.name);
        let inner_ty = p
            .condition()
            .map(|(_, _, t)| t.to_string())
            .unwrap_or_else(|| p.ty.clone());
        let is_bytes = inner_ty == "string" || inner_ty == "bytes";
        if let Some((_, bit, _)) = p.condition() {
            let _ = writeln!(out, "        if self.flags & (1u32 << {bit}) != 0 {{");
            let _ = writeln!(out, "            if let Some({fname}) = &self.{fname} {{");
            let _ = writeln!(out, "                {}", ctx.value_write(p, &fname, type_name, true));
            out.push_str("            }\n");
            out.push_str("        }\n");
        } else {
            let arg = if is_bytes {
                format!("&self.{fname}")
            } else {
                format!("self.{fname}")
            };
            let _ = writeln!(out, "        {}", ctx.value_write(p, &arg, type_name, false));
        }
    }
    out.push_str("    }\n}\n\n");
}

// ---------------- имена ----------------

/// SCREAMING_SNAKE: имя конструктора для константы id.
fn screaming(name: &str) -> String {
    let base = name.replace(['.', ' '], "_");
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in base.chars() {
        if ch.is_ascii_uppercase() && prev_lower {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
        prev_lower = ch.is_ascii_lowercase();
    }
    out
}

/// Имя типа результата: `messages.Dialogs` → `MessagesDialogs`,
/// `Set_client_DH_params_answer` → `Set_client_DH_params_answer` →
/// `SetClientDHParamsAnswer`.
fn rust_type_name(name: &str) -> String {
    // Vector<...> у функций не генерируется как тип; сохраняем строку.
    let base = name.replace(['.', ' '], "_");
    pascal_case(&base)
}

/// PascalCase с сохранением CAPS-частей: `server_DH_params_ok` →
/// `ServerDHParamsOk`, `resPQ` → `ResPQ`, `getUsers` → `GetUsers`.
fn pascal_case(name: &str) -> String {
    let mut out = String::new();
    for word in name.split(['_', '.', ' ']) {
        let mut w = word.to_string();
        if let Some(first) = w.chars().next() {
            let upper = first.to_uppercase().to_string();
            w.replace_range(0..1, &upper);
        }
        if !w.is_empty() {
            out.push_str(&w);
        }
    }
    out
}

/// snake_case имя поля + защита ключевых слов Rust.
fn field_name(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "type", "ref", "fn", "box", "final", "self", "super", "extern", "crate", "mod", "in",
        "as", "match", "mut", "pub", "use", "where", "if", "else", "loop", "while", "for",
        "return", "break", "continue", "const", "static", "enum", "struct", "impl", "trait",
        "let", "move", "unsafe", "dyn", "async", "await", "abstract", "become", "do", "priv",
        "typeof", "unsized", "virtual", "yield", "try", "true", "false",
    ];
    let snake: String = name
        .chars()
        .flat_map(|ch| {
            if ch.is_ascii_uppercase() {
                vec!['_', ch.to_ascii_lowercase()]
            } else {
                vec![ch]
            }
        })
        .collect();
    let snake = snake.trim_start_matches('_').to_string();
    if KEYWORDS.contains(&snake.as_str()) {
        format!("r#{snake}")
    } else {
        snake
    }
}

// ---------------- прелюдия генерата ----------------

const PRELUDE: &str = r#"
use tl_reader::{read_double, read_int, read_int128, read_int256, read_long, read_string,
    read_vector, Eof, Int128, Int256, Reader};
use tl_writer::{write_double, write_int, write_int128, write_int256, write_long, write_string,
    Writer};

/// Bare-чтение значения из TL-потока.
pub trait TlRead: Sized {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof>;
}

/// Bare-запись значения в TL-поток.
pub trait TlWrite {
    fn tl_write(&self, w: &mut Writer);
}

/// Boxed-чтение: конструктор уже прочитан и передан аргументом.
pub trait TlBoxedRead: Sized {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof>;
}

/// Boxed-запись: конструктор пишет реализация.
pub trait TlBoxedWrite {
    fn tl_write_boxed(&self, w: &mut Writer);
}

impl TlRead for i32 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int(r) }
}
impl TlWrite for i32 {
    fn tl_write(&self, w: &mut Writer) { write_int(w, *self) }
}
impl TlRead for u32 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { r.get() }
}
impl TlWrite for u32 {
    fn tl_write(&self, w: &mut Writer) { w.put(*self) }
}
impl TlRead for u64 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_long(r) }
}
impl TlWrite for u64 {
    fn tl_write(&self, w: &mut Writer) { write_long(w, *self) }
}
impl TlRead for f64 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_double(r) }
}
impl TlWrite for f64 {
    fn tl_write(&self, w: &mut Writer) { write_double(w, *self) }
}
impl TlRead for Vec<u8> {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_string(r) }
}
impl TlWrite for Vec<u8> {
    fn tl_write(&self, w: &mut Writer) { write_string(w, self) }
}
impl TlRead for Int128 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int128(r) }
}
impl TlWrite for Int128 {
    fn tl_write(&self, w: &mut Writer) { write_int128(w, *self) }
}
impl TlRead for Int256 {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { read_int256(r) }
}
impl TlWrite for Int256 {
    fn tl_write(&self, w: &mut Writer) { write_int256(w, *self) }
}
impl TlRead for bool {
    /// Bare bool: слово 0/1.
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { Ok(read_int(r)? != 0) }
}
impl TlWrite for bool {
    fn tl_write(&self, w: &mut Writer) { write_int(w, i32::from(*self)) }
}
impl<T: TlRead> TlRead for Vec<T> {
    /// Bare vector: счётчик и элементы, без конструктора 0x1cb5c415
    /// (как `Vector<long>` в схеме).
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> {
        read_vector(r, T::tl_read)
    }
}
impl<T: TlWrite> TlWrite for Vec<T> {
    fn tl_write(&self, w: &mut Writer) {
        w.put(self.len() as u32);
        for x in self {
            x.tl_write(w);
        }
    }
}
impl<T: TlBoxedRead> TlBoxedRead for Vec<T> {
    /// Boxed vector: конструктор уже прочитан; счётчик, затем
    /// boxed-элементы (у каждого свой конструктор).
    fn tl_read_boxed(r: &mut Reader, _cons: u32) -> Result<Self, Eof> {
        let count = r.get()? as usize;
        if !r.has(count) {
            return Err(Eof);
        }
        let mut v = Vec::with_capacity(count.min(r.remaining_primes()));
        for _ in 0..count {
            let cons = r.get()?;
            v.push(T::tl_read_boxed(r, cons)?);
        }
        Ok(v)
    }
}
impl<T: TlRead> TlRead for Box<T> {
    fn tl_read(r: &mut Reader) -> Result<Self, Eof> { Ok(Box::new(T::tl_read(r)?)) }
}
impl<T: TlWrite> TlWrite for Box<T> {
    fn tl_write(&self, w: &mut Writer) { T::tl_write(self, w) }
}
impl<T: TlBoxedRead> TlBoxedRead for Box<T> {
    fn tl_read_boxed(r: &mut Reader, cons: u32) -> Result<Self, Eof> {
        Ok(Box::new(T::tl_read_boxed(r, cons)?))
    }
}
impl<T: TlBoxedWrite> TlBoxedWrite for Box<T> {
    fn tl_write_boxed(&self, w: &mut Writer) { T::tl_write_boxed(self, w) }
}
impl<T: TlBoxedWrite> TlBoxedWrite for Vec<T> {
    fn tl_write_boxed(&self, w: &mut Writer) {
        w.put(tl_reader::ID_VECTOR);
        w.put(self.len() as u32);
        for x in self {
            x.tl_write_boxed(w);
        }
    }
}
"#;

