use crate::clean_file::CleanFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Int(i64),
    Double(String),
    Pixels(i32),
    String(String),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Semicolon,
    Comma,
    Dot,
    NumberSign, // '#'
    Plus,
    Minus,
    Equals,
    Or, // '|'
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub original: String,
    pub has_left_whitespace: bool,
}

pub struct Tokenizer {
    tokens: Vec<Token>,
    pos: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Result<Self, String> {
        let clean = CleanFile::parse(input)
            .map_err(|e| format!("Failed to clean comments: {:?}", e))?;
        let tokens = tokenize(&clean.content)?;
        Ok(Self { tokens, pos: 0 })
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    pub fn put_back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn expect_token(&mut self, expected: TokenType) -> Result<(), String> {
        match self.next() {
            Some(t) if t.token_type == expected => Ok(()),
            Some(t) => Err(format!("Expected {:?}, got {:?}", expected, t.token_type)),
            None => Err(format!("Expected {:?}, got EOF", expected)),
        }
    }

    pub fn expect_name(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token { token_type: TokenType::Name(name), .. }) => Ok(name),
            Some(t) => Err(format!("Expected Name, got {:?}", t.token_type)),
            None => Err("Expected Name, got EOF".to_string()),
        }
    }

    pub fn expect_string(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token { token_type: TokenType::String(s), .. }) => Ok(s),
            Some(t) => Err(format!("Expected String literal, got {:?}", t.token_type)),
            None => Err("Expected String literal, got EOF".to_string()),
        }
    }
}

fn is_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

fn is_name_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_'
}

fn tokenize(content: &str) -> Result<Vec<Token>, String> {
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut tokens = Vec::new();

    while i < len {
        let mut had_whitespace = false;
        while i < len && bytes[i].is_ascii_whitespace() {
            had_whitespace = true;
            i += 1;
        }
        if i >= len {
            break;
        }

        let ch = bytes[i];

        // String literal
        if ch == b'"' {
            let start = i;
            i += 1;
            let mut s = String::new();
            let mut closed = false;
            while i < len {
                let c = bytes[i];
                if c == b'"' {
                    i += 1;
                    closed = true;
                    break;
                } else if c == b'\\' {
                    i += 1;
                    if i >= len {
                        return Err("Unterminated escape sequence in string".to_string());
                    }
                    match bytes[i] {
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        other => {
                            s.push('\\');
                            s.push(other as char);
                        }
                    }
                    i += 1;
                } else {
                    s.push(c as char);
                    i += 1;
                }
            }
            if !closed {
                return Err("Unterminated string literal".to_string());
            }
            let orig = std::str::from_utf8(&bytes[start..i]).unwrap().to_string();
            tokens.push(Token {
                token_type: TokenType::String(s),
                original: orig,
                has_left_whitespace: had_whitespace,
            });
            continue;
        }

        // Single punctuation tokens
        let single = match ch {
            b'(' => Some(TokenType::LeftParen),
            b')' => Some(TokenType::RightParen),
            b'{' => Some(TokenType::LeftBrace),
            b'}' => Some(TokenType::RightBrace),
            b'[' => Some(TokenType::LeftBracket),
            b']' => Some(TokenType::RightBracket),
            b':' => Some(TokenType::Colon),
            b';' => Some(TokenType::Semicolon),
            b',' => Some(TokenType::Comma),
            b'#' => Some(TokenType::NumberSign),
            b'+' => Some(TokenType::Plus),
            b'-' => Some(TokenType::Minus),
            b'=' => Some(TokenType::Equals),
            b'|' => Some(TokenType::Or),
            _ => None,
        };

        if let Some(tok) = single {
            tokens.push(Token {
                token_type: tok,
                original: (ch as char).to_string(),
                has_left_whitespace: had_whitespace,
            });
            i += 1;
            continue;
        }

        // Check for Dot or Number starting with dot (e.g. .5)
        if ch == b'.' {
            if i + 1 < len && is_digit(bytes[i + 1]) {
                let start = i;
                i += 1;
                while i < len && is_digit(bytes[i]) {
                    i += 1;
                }
                let s = std::str::from_utf8(&bytes[start..i]).unwrap();
                tokens.push(Token {
                    token_type: TokenType::Double(s.to_string()),
                    original: s.to_string(),
                    has_left_whitespace: had_whitespace,
                });
                continue;
            } else {
                tokens.push(Token {
                    token_type: TokenType::Dot,
                    original: ".".to_string(),
                    has_left_whitespace: had_whitespace,
                });
                i += 1;
                continue;
            }
        }

        // Number or Identifier / Name
        if is_name_char(ch) {
            let start = i;
            let mut all_digits = true;
            while i < len && is_name_char(bytes[i]) {
                if !is_digit(bytes[i]) {
                    all_digits = false;
                }
                i += 1;
            }

            let word = std::str::from_utf8(&bytes[start..i]).unwrap();

            // Check if it was digits followed by 'px'
            if word.ends_with("px") && word.len() > 2 && word[..word.len() - 2].chars().all(|c| c.is_ascii_digit()) {
                let val: i32 = word[..word.len() - 2].parse().map_err(|e| format!("Bad px: {}", e))?;
                tokens.push(Token {
                    token_type: TokenType::Pixels(val),
                    original: word.to_string(),
                    has_left_whitespace: had_whitespace,
                });
                continue;
            }

            if all_digits {
                // Check if followed immediately by '.' and more digits (forming a double)
                if i < len && bytes[i] == b'.' && (i + 1 < len && is_digit(bytes[i + 1])) {
                    i += 1; // skip '.'
                    while i < len && is_digit(bytes[i]) {
                        i += 1;
                    }
                    let double_str = std::str::from_utf8(&bytes[start..i]).unwrap();
                    tokens.push(Token {
                        token_type: TokenType::Double(double_str.to_string()),
                        original: double_str.to_string(),
                        has_left_whitespace: had_whitespace,
                    });
                    continue;
                }

                let val: i64 = word.parse().map_err(|e| format!("Bad int: {}", e))?;
                tokens.push(Token {
                    token_type: TokenType::Int(val),
                    original: word.to_string(),
                    has_left_whitespace: had_whitespace,
                });
            } else {
                tokens.push(Token {
                    token_type: TokenType::Name(word.to_string()),
                    original: word.to_string(),
                    has_left_whitespace: had_whitespace,
                });
            }
            continue;
        }

        return Err(format!("Unexpected character '{}' (0x{:02x}) at index {}", ch as char, ch, i));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_basic() {
        let input = "windowMinWidth: 380px; font: font(13px); align: align(top); // comment";
        let mut tok = Tokenizer::new(input).unwrap();
        assert_eq!(tok.next().unwrap().token_type, TokenType::Name("windowMinWidth".into()));
        assert_eq!(tok.next().unwrap().token_type, TokenType::Colon);
        assert_eq!(tok.next().unwrap().token_type, TokenType::Pixels(380));
        assert_eq!(tok.next().unwrap().token_type, TokenType::Semicolon);
    }
}
