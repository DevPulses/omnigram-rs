use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::tokenizer::{Tokenizer, TokenType};
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Double(f64),
    Pixels(i32),
    Bool(bool),
    String(String),
    Color(String), // Name of the color
    Point(Point),
    Size(Size),
    Margins(Margins),
    Align(Align),
    Font(Font),
    Icon(Icon),
    Struct {
        struct_name: String,
        fields: HashMap<String, Value>,
    },
    Reference(String),
}

#[derive(Debug, Clone)]
pub struct StructFieldDef {
    pub name: String,
    pub field_type: String, // e.g. "pixels", "color", "font", or another struct name
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructFieldDef>,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedModule {
    pub path: PathBuf,
    pub includes: Vec<String>,
    pub struct_defs: HashMap<String, StructDef>,
    pub variables: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct StyleSheet {
    pub modules: HashMap<PathBuf, ParsedModule>,
    pub all_variables: HashMap<String, Value>,
    pub all_struct_defs: HashMap<String, StructDef>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load root style or palette file and transitively parse all `using` files.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P, search_paths: &[PathBuf]) -> Result<(), String> {
        let abs_path = path.as_ref().canonicalize().map_err(|e| format!("Cannot resolve path {:?}: {}", path.as_ref(), e))?;
        if self.modules.contains_key(&abs_path) {
            return Ok(());
        }

        let content = std::fs::read_to_string(&abs_path)
            .map_err(|e| format!("Cannot read file {:?}: {}", abs_path, e))?;

        let mut module = ParsedModule {
            path: abs_path.clone(),
            ..Default::default()
        };

        module.parse(&content)?;

        // Transitively load included files
        for inc in &module.includes {
            let found_path = find_include_file(inc, search_paths, abs_path.parent().unwrap())?;
            self.load_file(found_path, search_paths)?;
        }

        // Register structs and variables
        for (name, sdef) in &module.struct_defs {
            self.all_struct_defs.insert(name.clone(), sdef.clone());
        }

        for (name, val) in &module.variables {
            self.all_variables.insert(name.clone(), val.clone());
        }

        self.modules.insert(abs_path, module);
        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.all_variables.get(name)
    }

    pub fn get_pixels(&self, name: &str) -> Option<i32> {
        match self.get_variable(name) {
            Some(Value::Pixels(px)) => Some(*px),
            Some(Value::Int(i)) => Some(*i as i32),
            _ => None,
        }
    }
}

fn find_include_file(name: &str, search_paths: &[PathBuf], current_dir: &Path) -> Result<PathBuf, String> {
    let direct = current_dir.join(name);
    if direct.exists() {
        return Ok(direct);
    }

    for dir in search_paths {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!("Could not find include file '{}' in search paths", name))
}

impl ParsedModule {
    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        let mut tokenizer = Tokenizer::new(content)?;

        while !tokenizer.at_end() {
            let name_tok = match tokenizer.next() {
                Some(tok) => tok,
                None => break,
            };

            match name_tok.token_type {
                TokenType::Name(ref keyword) if keyword == "using" => {
                    let inc_file = tokenizer.expect_string()?;
                    tokenizer.expect_token(TokenType::Semicolon)?;
                    self.includes.push(inc_file);
                }
                TokenType::Name(ident) => {
                    let next = match tokenizer.next() {
                        Some(t) => t,
                        None => return Err(format!("Unexpected EOF after identifier '{}'", ident)),
                    };

                    match next.token_type {
                        TokenType::LeftBrace => {
                            // Struct definition: StructName { field: type; ... }
                            let struct_def = parse_struct_body(ident.clone(), &mut tokenizer)?;
                            self.struct_defs.insert(ident, struct_def);
                        }
                        TokenType::Colon => {
                            // Variable definition: varName: value;
                            let val = parse_value(&mut tokenizer, &self.variables, &self.struct_defs)?;
                            if !matches!(val, Value::Struct { .. }) {
                                tokenizer.expect_token(TokenType::Semicolon)?;
                            } else {
                                // Struct literals might end with or without semicolon in .style
                                if let Some(peek) = tokenizer.peek() {
                                    if peek.token_type == TokenType::Semicolon {
                                        tokenizer.next();
                                    }
                                }
                            }
                            self.variables.insert(ident, val);
                        }
                        _ => {
                            return Err(format!("Expected '{{' or ':' after '{}', got {:?}", ident, next.token_type));
                        }
                    }
                }
                _ => return Err(format!("Unexpected token at top level: {:?}", name_tok.token_type)),
            }
        }

        Ok(())
    }
}

fn parse_struct_body(name: String, tokenizer: &mut Tokenizer) -> Result<StructDef, String> {
    let mut fields = Vec::new();

    while !tokenizer.at_end() {
        if let Some(peek) = tokenizer.peek() {
            if peek.token_type == TokenType::RightBrace {
                tokenizer.next(); // consume '}'
                break;
            }
        }

        let field_name = tokenizer.expect_name()?;
        tokenizer.expect_token(TokenType::Colon)?;
        let field_type = tokenizer.expect_name()?;
        tokenizer.expect_token(TokenType::Semicolon)?;

        fields.push(StructFieldDef {
            name: field_name,
            field_type,
        });
    }

    Ok(StructDef { name, fields })
}

fn parse_value(
    tokenizer: &mut Tokenizer,
    variables: &HashMap<String, Value>,
    struct_defs: &HashMap<String, StructDef>,
) -> Result<Value, String> {
    let token = tokenizer.next().ok_or_else(|| "Unexpected EOF reading value".to_string())?;

    match token.token_type {
        TokenType::Int(val) => {
            let mut result = val;
            while let Some(peek) = tokenizer.peek() {
                if peek.token_type == TokenType::Plus {
                    tokenizer.next();
                    let next_val = parse_int_expr(tokenizer)?;
                    result += next_val;
                } else if peek.token_type == TokenType::Minus {
                    tokenizer.next();
                    let next_val = parse_int_expr(tokenizer)?;
                    result -= next_val;
                } else {
                    break;
                }
            }
            Ok(Value::Int(result))
        }
        TokenType::Pixels(px) => {
            let mut result = px;
            while let Some(peek) = tokenizer.peek() {
                if peek.token_type == TokenType::Plus {
                    tokenizer.next();
                    let next_val = parse_px_expr(tokenizer)?;
                    result += next_val;
                } else if peek.token_type == TokenType::Minus {
                    tokenizer.next();
                    let next_val = parse_px_expr(tokenizer)?;
                    result -= next_val;
                } else {
                    break;
                }
            }
            Ok(Value::Pixels(result))
        }
        TokenType::Minus => {
            // Negative number / px
            let next_tok = tokenizer.next().ok_or("Expected number after '-'")?;
            match next_tok.token_type {
                TokenType::Int(i) => Ok(Value::Int(-i)),
                TokenType::Pixels(px) => Ok(Value::Pixels(-px)),
                _ => Err("Invalid token after unary minus".to_string()),
            }
        }
        TokenType::Double(s) => {
            let d: f64 = s.parse().map_err(|e| format!("Bad double: {}", e))?;
            Ok(Value::Double(d))
        }
        TokenType::String(s) => Ok(Value::String(s)),
        TokenType::Name(ref name) if name == "true" => Ok(Value::Bool(true)),
        TokenType::Name(ref name) if name == "false" => Ok(Value::Bool(false)),
        TokenType::Name(ref name) if name == "point" => {
            tokenizer.expect_token(TokenType::LeftParen)?;
            let x = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::Comma)?;
            let y = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::RightParen)?;
            Ok(Value::Point(Point { x, y }))
        }
        TokenType::Name(ref name) if name == "size" => {
            tokenizer.expect_token(TokenType::LeftParen)?;
            let width = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::Comma)?;
            let height = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::RightParen)?;
            Ok(Value::Size(Size { width, height }))
        }
        TokenType::Name(ref name) if name == "margins" => {
            tokenizer.expect_token(TokenType::LeftParen)?;
            let left = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::Comma)?;
            let top = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::Comma)?;
            let right = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::Comma)?;
            let bottom = parse_px_expr(tokenizer)?;
            tokenizer.expect_token(TokenType::RightParen)?;
            Ok(Value::Margins(Margins { left, top, right, bottom }))
        }
        TokenType::Name(ref name) if name == "align" => {
            tokenizer.expect_token(TokenType::LeftParen)?;
            let align_str = tokenizer.expect_name()?;
            tokenizer.expect_token(TokenType::RightParen)?;
            let align = Align::parse(&align_str)
                .ok_or_else(|| format!("Unknown align: {}", align_str))?;
            Ok(Value::Align(align))
        }
        TokenType::Name(ref name) if name == "font" => {
            tokenizer.expect_token(TokenType::LeftParen)?;
            let mut family = String::new();
            let mut size = 0;
            let mut flags = FontFlags::empty();

            while !tokenizer.at_end() {
                if let Some(peek) = tokenizer.peek() {
                    if peek.token_type == TokenType::RightParen {
                        tokenizer.next();
                        break;
                    }
                }

                let param_tok = tokenizer.next().ok_or("Unexpected EOF in font(...)")?;
                match param_tok.token_type {
                    TokenType::Name(ref f) if f == "bold" || f == "semibold" => flags |= FontFlags::BOLD,
                    TokenType::Name(ref f) if f == "italic" => flags |= FontFlags::ITALIC,
                    TokenType::Name(ref f) if f == "underline" => flags |= FontFlags::UNDERLINE,
                    TokenType::Pixels(px) => size = px,
                    TokenType::String(fam) => family = fam,
                    _ => return Err(format!("Unexpected parameter in font: {:?}", param_tok.token_type)),
                }
            }

            Ok(Value::Font(Font { family, size, flags }))
        }
        TokenType::Name(ref name) if name == "icon" => {
            let mut parts = Vec::new();

            if let Some(peek) = tokenizer.peek() {
                if peek.token_type == TokenType::LeftBrace {
                    tokenizer.next(); // complex icon
                    while !tokenizer.at_end() {
                        if let Some(p) = tokenizer.peek() {
                            if p.token_type == TokenType::RightBrace {
                                tokenizer.next();
                                break;
                            }
                        }
                        tokenizer.expect_token(TokenType::LeftBrace)?;
                        let mono = parse_mono_icon(tokenizer)?;
                        tokenizer.expect_token(TokenType::RightBrace)?;
                        parts.push(mono);
                        if let Some(p) = tokenizer.peek() {
                            if p.token_type == TokenType::Comma {
                                tokenizer.next();
                            }
                        }
                    }
                } else if peek.token_type == TokenType::LeftParen {
                    tokenizer.next(); // short icon
                    let mono = parse_mono_icon(tokenizer)?;
                    tokenizer.expect_token(TokenType::RightParen)?;
                    parts.push(mono);
                }
            }

            Ok(Value::Icon(Icon { parts }))
        }
        TokenType::Name(name) => {
            // Could be struct instantiation: StructName { ... } or StructName(Parent) { ... }
            // Or reference to another variable/color: varName or Struct.field
            if let Some(peek) = tokenizer.peek() {
                if peek.token_type == TokenType::LeftParen || peek.token_type == TokenType::LeftBrace {
                    // Struct instantiation!
                    let struct_name = name;
                    let mut parent_fields = HashMap::new();

                    if tokenizer.peek().unwrap().token_type == TokenType::LeftParen {
                        tokenizer.next(); // consume '('
                        let parent_name = tokenizer.expect_name()?;
                        tokenizer.expect_token(TokenType::RightParen)?;

                        // Inherit parent fields if parent exists
                        if let Some(Value::Struct { fields, .. }) = variables.get(&parent_name) {
                            parent_fields = fields.clone();
                        }
                    }

                    tokenizer.expect_token(TokenType::LeftBrace)?;
                    let mut fields = parent_fields;

                    while !tokenizer.at_end() {
                        if let Some(p) = tokenizer.peek() {
                            if p.token_type == TokenType::RightBrace {
                                tokenizer.next();
                                break;
                            }
                        }

                        let f_name = tokenizer.expect_name()?;
                        tokenizer.expect_token(TokenType::Colon)?;
                        let f_val = parse_value(tokenizer, variables, struct_defs)?;
                        tokenizer.expect_token(TokenType::Semicolon)?;

                        fields.insert(f_name, f_val);
                    }

                    return Ok(Value::Struct {
                        struct_name,
                        fields,
                    });
                } else if peek.token_type == TokenType::Dot {
                    // Access to field like `other.field`
                    tokenizer.next(); // consume '.'
                    let field = tokenizer.expect_name()?;
                    return Ok(Value::Reference(format!("{}.{}", name, field)));
                }
            }

            // Simple reference or color name
            Ok(Value::Reference(name))
        }
        _ => Err(format!("Unexpected token parsing value: {:?}", token.token_type)),
    }
}

fn parse_mono_icon(tokenizer: &mut Tokenizer) -> Result<MonoIcon, String> {
    let filename = tokenizer.expect_string()?;
    tokenizer.expect_token(TokenType::Comma)?;
    let color_name = tokenizer.expect_name()?;

    let mut padding = Margins::default();
    if let Some(peek) = tokenizer.peek() {
        if peek.token_type == TokenType::Comma {
            tokenizer.next();
            // Could be point(...) or margins(...)
            let tok = tokenizer.next().ok_or("Expected offset after comma in icon")?;
            match tok.token_type {
                TokenType::Name(ref n) if n == "point" => {
                    tokenizer.expect_token(TokenType::LeftParen)?;
                    let x = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::Comma)?;
                    let y = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::RightParen)?;
                    padding = Margins { left: x, top: y, right: 0, bottom: 0 };
                }
                TokenType::Name(ref n) if n == "margins" => {
                    tokenizer.expect_token(TokenType::LeftParen)?;
                    let l = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::Comma)?;
                    let t = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::Comma)?;
                    let r = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::Comma)?;
                    let b = parse_px_expr(tokenizer)?;
                    tokenizer.expect_token(TokenType::RightParen)?;
                    padding = Margins { left: l, top: t, right: r, bottom: b };
                }
                _ => return Err("Expected point or margins for icon offset".to_string()),
            }
        }
    }

    Ok(MonoIcon {
        filename,
        color: color_name,
        padding,
    })
}

fn parse_px_expr(tokenizer: &mut Tokenizer) -> Result<i32, String> {
    let tok = tokenizer.next().ok_or("Expected px value")?;
    match tok.token_type {
        TokenType::Pixels(px) => Ok(px),
        TokenType::Int(i) => Ok(i as i32),
        _ => Err(format!("Expected pixels or int, got {:?}", tok.token_type)),
    }
}

fn parse_int_expr(tokenizer: &mut Tokenizer) -> Result<i64, String> {
    let tok = tokenizer.next().ok_or("Expected int value")?;
    match tok.token_type {
        TokenType::Int(i) => Ok(i),
        TokenType::Pixels(px) => Ok(px as i64),
        _ => Err(format!("Expected int or pixels, got {:?}", tok.token_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_module() {
        let input = r#"
            using "ui/colors.palette";

            TextStyle {
                font: font;
                lineHeight: pixels;
            }

            defaultTextStyle: TextStyle {
                font: font(13px);
                lineHeight: 18px;
            }

            windowMinWidth: 380px;
        "#;

        let mut module = ParsedModule::default();
        module.parse(input).unwrap();

        assert_eq!(module.includes, vec!["ui/colors.palette"]);
        assert!(module.struct_defs.contains_key("TextStyle"));
        assert_eq!(module.variables.get("windowMinWidth"), Some(&Value::Pixels(380)));

        if let Some(Value::Struct { struct_name, fields }) = module.variables.get("defaultTextStyle") {
            assert_eq!(struct_name, "TextStyle");
            assert_eq!(fields.get("lineHeight"), Some(&Value::Pixels(18)));
        } else {
            panic!("Expected struct defaultTextStyle");
        }
    }
}
