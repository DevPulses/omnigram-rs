use std::collections::HashMap;
use crate::tokenizer::{Tokenizer, TokenType};
use crate::types::Color;

#[derive(Debug, Clone, Default)]
pub struct Palette {
    pub colors: HashMap<String, Color>,
}

impl Palette {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut tokenizer = Tokenizer::new(input)?;
        let mut colors: HashMap<String, Color> = HashMap::new();

        while !tokenizer.at_end() {
            let name = match tokenizer.expect_name() {
                Ok(n) => n,
                Err(_) if tokenizer.at_end() => break,
                Err(e) => return Err(e),
            };

            tokenizer.expect_token(TokenType::Colon)?;

            // Color value: either '#hex', 'transparent', or another color name
            let color = match tokenizer.next() {
                Some(tok) => match tok.token_type {
                    TokenType::NumberSign => {
                        // Could be followed by Int or Name (e.g. #ffffff or #00000018 or #fff)
                        let next_tok = tokenizer.next().ok_or("Unexpected EOF after '#' in color")?;
                        let hex_str = next_tok.original;
                        let (r, g, b, a) = parse_hex_color(&hex_str)?;
                        
                        // Check for fallback '|'
                        let mut fallback = None;
                        if let Some(peek) = tokenizer.peek() {
                            if peek.token_type == TokenType::Or {
                                tokenizer.next(); // consume '|'
                                let fb_name = tokenizer.expect_name()?;
                                fallback = Some(fb_name);
                            }
                        }

                        Color { r, g, b, a, fallback }
                    }
                    TokenType::Name(ref n) if n == "transparent" => {
                        Color { r: 255, g: 255, b: 255, a: 0, fallback: None }
                    }
                    TokenType::Name(ref other_name) => {
                        // Reference to another color
                        if let Some(existing) = colors.get(other_name) {
                            existing.clone()
                        } else {
                            // Forward reference or fallback reference
                            Color { r: 0, g: 0, b: 0, a: 255, fallback: Some(other_name.clone()) }
                        }
                    }
                    _ => return Err(format!("Unexpected token for color: {:?}", tok.token_type)),
                },
                None => return Err("Unexpected EOF parsing color value".to_string()),
            };

            tokenizer.expect_token(TokenType::Semicolon)?;
            colors.insert(name, color);
        }

        // Resolve references if fallback is used as value reference
        resolve_color_references(&mut colors);

        Ok(Self { colors })
    }

    pub fn get(&self, name: &str) -> Option<&Color> {
        self.colors.get(name)
    }
}

fn parse_hex_color(s: &str) -> Result<(u8, u8, u8, u8), String> {
    let s = s.trim();
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
        Ok((r, g, b, 255))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
        let a = u8::from_str_radix(&s[6..8], 16).map_err(|e| e.to_string())?;
        Ok((r, g, b, a))
    } else {
        Err(format!("Invalid hex color length: '{}' ({})", s, s.len()))
    }
}

fn resolve_color_references(colors: &mut HashMap<String, Color>) {
    let names: Vec<String> = colors.keys().cloned().collect();
    for name in names {
        if let Some(col) = colors.get(&name) {
            if col.r == 0 && col.g == 0 && col.b == 0 && col.a == 255 {
                if let Some(ref ref_name) = col.fallback {
                    if let Some(target) = colors.get(ref_name).cloned() {
                        colors.insert(name, target);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_palette() {
        let input = r#"
            windowBg: #ffffff;
            windowFg: #000000;
            windowFgOver: windowFg;
            shadowFg: #00000018;
            transparentBg: transparent;
        "#;
        let pal = Palette::parse(input).unwrap();
        assert_eq!(pal.get("windowBg").unwrap(), &Color { r: 255, g: 255, b: 255, a: 255, fallback: None });
        assert_eq!(pal.get("windowFg").unwrap(), &Color { r: 0, g: 0, b: 0, a: 255, fallback: None });
        assert_eq!(pal.get("windowFgOver").unwrap(), &Color { r: 0, g: 0, b: 0, a: 255, fallback: None });
        assert_eq!(pal.get("shadowFg").unwrap(), &Color { r: 0, g: 0, b: 0, a: 0x18, fallback: None });
        assert_eq!(pal.get("transparentBg").unwrap(), &Color { r: 255, g: 255, b: 255, a: 0, fallback: None });
    }
}
