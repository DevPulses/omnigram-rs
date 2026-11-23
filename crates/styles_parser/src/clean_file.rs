/// Strip single-line (//) and multi-line (/* */) comments,
/// matching codegen/common/clean_file.cpp.
pub struct CleanFile {
    pub content: String,
    pub single_line_comments: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CleanFileError {
    UnterminatedMultiLineComment,
}

impl CleanFile {
    pub fn parse(raw: &str) -> Result<Self, CleanFileError> {
        let mut result = String::with_capacity(raw.len());
        let mut single_line_comments: Vec<String> = Vec::new();

        let bytes = raw.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        enum InsideComment {
            None,
            SingleLine,
            MultiLine,
        }

        let mut inside_comment = InsideComment::None;
        let mut inside_string = false;
        let mut line_number = 0;

        let mut comment_buf = String::new();

        while i < len {
            let ch = bytes[i];
            let next_ch = if i + 1 < len { bytes[i + 1] } else { 0 };

            if matches!(inside_comment, InsideComment::None) && ch == b'"' {
                let escaped = (i > 0 && bytes[i - 1] == b'\\')
                    && !(i > 1 && bytes[i - 2] == b'\\');
                if !escaped {
                    inside_string = !inside_string;
                }
            }

            if inside_string {
                if ch == b'\n' {
                    line_number += 1;
                }
                result.push(ch as char);
                i += 1;
                continue;
            }

            match inside_comment {
                InsideComment::None => {
                    if ch == b'/' && next_ch == b'/' {
                        inside_comment = InsideComment::SingleLine;
                        comment_buf.clear();
                        i += 2;
                    } else if ch == b'/' && next_ch == b'*' {
                        inside_comment = InsideComment::MultiLine;
                        i += 2;
                    } else {
                        if ch == b'\n' {
                            line_number += 1;
                        }
                        result.push(ch as char);
                        i += 1;
                    }
                }
                InsideComment::SingleLine => {
                    if ch == b'\r' && next_ch == b'\n' {
                        while single_line_comments.len() <= line_number {
                            single_line_comments.push(String::new());
                        }
                        single_line_comments[line_number] = comment_buf.clone();
                        comment_buf.clear();
                        result.push(' ');
                        i += 2;
                        line_number += 1;
                        result.push('\r');
                        result.push('\n');
                        inside_comment = InsideComment::None;
                    } else if ch == b'\n' {
                        while single_line_comments.len() <= line_number {
                            single_line_comments.push(String::new());
                        }
                        single_line_comments[line_number] = comment_buf.clone();
                        comment_buf.clear();
                        result.push(' ');
                        i += 1;
                        line_number += 1;
                        result.push('\n');
                        inside_comment = InsideComment::None;
                    } else {
                        comment_buf.push(ch as char);
                        i += 1;
                    }
                }
                InsideComment::MultiLine => {
                    if ch == b'*' && next_ch == b'/' {
                        result.push(' ');
                        i += 2;
                        inside_comment = InsideComment::None;
                    } else if ch == b'\r' && next_ch == b'\n' {
                        result.push(' ');
                        i += 2;
                        line_number += 1;
                        result.push('\r');
                        result.push('\n');
                    } else if ch == b'\n' {
                        result.push(' ');
                        i += 1;
                        line_number += 1;
                        result.push('\n');
                    } else {
                        i += 1;
                    }
                }
            }
        }

        if matches!(inside_comment, InsideComment::MultiLine) {
            return Err(CleanFileError::UnterminatedMultiLineComment);
        }

        if matches!(inside_comment, InsideComment::SingleLine) {
            while single_line_comments.len() <= line_number {
                single_line_comments.push(String::new());
            }
            single_line_comments[line_number] = comment_buf;
            result.push(' ');
        }

        Ok(Self {
            content: result,
            single_line_comments,
        })
    }
}
