use std::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TK {
    String,
    Number,
    True,
    False,
    Nil,
    Ident,
    Fnc,
    Pub,
    Import,
    As,
    Return,
    Output,
    And,
    Or,
    Not,
    For,
    In,
    While,
    Repeat,
    Break,
    Continue,
    If,
    Else,
    Try,
    Catch,
    Lua,
    StrType,
    NumType,
    BoolType,
    ListType,
    MapType,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    DotDot,
    Pipe,
    Colon,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusEq,
    Arrow,
    Question,
    Eof,
    LuaBlock,
    OutputBlock,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TK,
    pub value: String,
    pub line: u32,
    pub col: u32,
}

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: u32,
    col: u32,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            line: 0,
            col: 0,
            tokens: Vec::new(),
        }
    }

    #[inline]
    fn cur_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline]
    fn peek_byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    #[inline]
    fn advance_char(&mut self) -> Option<char> {
        let ch = self.src[self.pos..].chars().next()?;
        let len = ch.len_utf8();
        self.pos += len;
        if ch == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += len as u32;
        }
        Some(ch)
    }

    #[inline]
    fn advance_byte(&mut self) {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            if b == b'\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.cur_byte() {
            match b {
                b' ' | b'\t' | b'\r' | b'\n' => self.advance_byte(),
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(b) = self.cur_byte() {
            self.advance_byte();
            if b == b'\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        while let Some(b) = self.cur_byte() {
            self.advance_byte();
            if b == b'*' && self.cur_byte() == Some(b'/') {
                self.advance_byte();
                return;
            }
        }
    }

    fn lex_string(&mut self, quote: char, start_line: u32, start_col: u32) {
        let mut val = String::new();
        while let Some(c) = self.advance_char() {
            if c == quote {
                break;
            }
            if c == '\\' {
                if let Some(esc) = self.advance_char() {
                    val.push(esc);
                }
                continue;
            }
            val.push(c);
        }
        self.emit(TK::String, val, start_line, start_col);
    }

    fn lex_number(&mut self, first: char, start_line: u32, start_col: u32) {
        let mut val = String::new();
        val.push(first);
        let mut has_dot = first == '.';
        while let Some(ch) = self.cur_byte().and_then(|b| Some(b as char)) {
            if ch.is_ascii_digit() {
                val.push(ch);
                self.advance_byte();
            } else if ch == '.' && !has_dot && self.peek_byte(1) != Some(b'.') {
                has_dot = true;
                val.push(ch);
                self.advance_byte();
            } else {
                break;
            }
        }
        self.emit(TK::Number, val, start_line, start_col);
    }

    fn lex_ident(&mut self, first: char, start_line: u32, start_col: u32) {
        let start = self.pos - first.len_utf8();
        while let Some(b) = self.cur_byte() {
            let ch = b as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance_byte();
            } else {
                break;
            }
        }
        let val = self.src[start..self.pos].to_string();
        let kind = match val.as_str() {
            "fnc" => TK::Fnc,
            "pub" => TK::Pub,
            "import" => TK::Import,
            "as" => TK::As,
            "return" => TK::Return,
            "output" => TK::Output,
            "and" => TK::And,
            "or" => TK::Or,
            "not" => TK::Not,
            "for" => TK::For,
            "in" => TK::In,
            "while" => TK::While,
            "repeat" => TK::Repeat,
            "break" => TK::Break,
            "continue" => TK::Continue,
            "if" => TK::If,
            "else" => TK::Else,
            "try" => TK::Try,
            "catch" => TK::Catch,
            "lua" => TK::Lua,
            "true" => TK::True,
            "false" => TK::False,
            "nil" => TK::Nil,
            "str" => TK::StrType,
            "num" => TK::NumType,
            "bool" => TK::BoolType,
            "list" => TK::ListType,
            "map" => TK::MapType,
            _ => TK::Ident,
        };

        self.emit(kind, val, start_line, start_col);

        if kind == TK::Lua || kind == TK::Output {
            self.skip_ws();
            if self.cur_byte() == Some(b'{') {
                self.advance_byte();
                let block_start = self.pos;

                let (end, _) = if kind == TK::Lua {
                    self.scan_lua_block()
                } else {
                    self.scan_output_block()
                };

                let content = if end > block_start {
                    self.src[block_start..end].to_string()
                } else {
                    String::new()
                };

                self.emit(
                    if kind == TK::Lua {
                        TK::LuaBlock
                    } else {
                        TK::OutputBlock
                    },
                    content,
                    start_line,
                    start_col,
                );
            }
        }
    }

    fn scan_lua_block(&mut self) -> (usize, bool) {
        let mut depth = 1usize;
        let n = self.bytes.len();

        while self.pos < n {
            let b = self.bytes[self.pos];
            if b == b'-' && self.pos + 1 < n && self.bytes[self.pos + 1] == b'-' {
                if self.pos + 3 < n
                    && self.bytes[self.pos + 2] == b'['
                    && self.bytes[self.pos + 3] == b'['
                {
                    self.pos += 4;
                    while self.pos + 1 < n
                        && !(self.bytes[self.pos] == b']' && self.bytes[self.pos + 1] == b']')
                    {
                        self.pos += 1;
                    }
                    self.pos += 2;
                    continue;
                }
                self.pos += 2;
                while self.pos < n && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            if b == b'"' || b == b'\'' {
                let q = b;
                self.pos += 1;
                while self.pos < n && self.bytes[self.pos] != q {
                    if self.bytes[self.pos] == b'\\' && self.pos + 1 < n {
                        self.pos += 1;
                    }
                    self.pos += 1;
                }
                self.pos += 1;
                continue;
            }
            if b == b'[' && self.pos + 1 < n && self.bytes[self.pos + 1] == b'[' {
                self.pos += 2;
                while self.pos + 1 < n
                    && !(self.bytes[self.pos] == b']' && self.bytes[self.pos + 1] == b']')
                {
                    self.pos += 1;
                }
                self.pos += 2;
                continue;
            }
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    let end = self.pos;
                    self.pos += 1;
                    return (end, true);
                }
            }
            self.pos += 1;
        }
        (self.pos, false)
    }

    fn scan_output_block(&mut self) -> (usize, bool) {
        let mut depth = 1usize;
        let n = self.bytes.len();
        while self.pos < n {
            let b = self.bytes[self.pos];
            if b == b'{' && (self.pos + 1 >= n || self.bytes[self.pos + 1] != b'|') {
                depth += 1;
            } else if b == b'}' && (self.pos == 0 || self.bytes[self.pos - 1] != b'|') {
                if depth > 0 {
                    depth -= 1;
                }
                if depth == 0 {
                    let end = self.pos;
                    self.pos += 1;
                    return (end, true);
                }
            }
            self.pos += 1;
        }
        (self.pos, false)
    }

    #[inline]
    fn emit(&mut self, kind: TK, value: String, line: u32, col: u32) {
        self.tokens.push(Token {
            kind,
            value,
            line,
            col,
        });
    }

    #[inline]
    fn emit_simple(&mut self, kind: TK, value: &str, line: u32, col: u32) {
        self.tokens.push(Token {
            kind,
            value: value.to_string(),
            line,
            col,
        });
    }

    fn run(mut self) -> Vec<Token> {
        while let Some(b) = self.cur_byte() {
            // whitespace
            if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
                self.advance_byte();
                continue;
            }

            // comments
            if b == b'/' && self.peek_byte(1) == Some(b'/') {
                self.advance_byte();
                self.advance_byte();
                self.skip_line_comment();
                continue;
            }
            if b == b'/' && self.peek_byte(1) == Some(b'*') {
                self.advance_byte();
                self.advance_byte();
                self.skip_block_comment();
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;
            let c = self.advance_char().unwrap();

            if c == '"' || c == '\'' {
                self.lex_string(c, start_line, start_col);
                continue;
            }

            if c.is_ascii_digit() {
                self.lex_number(c, start_line, start_col);
                continue;
            }

            if c.is_ascii_alphabetic() || c == '_' {
                self.lex_ident(c, start_line, start_col);
                continue;
            }

            match c {
                '{' => self.emit_simple(TK::LBrace, "{", start_line, start_col),
                '}' => self.emit_simple(TK::RBrace, "}", start_line, start_col),
                '(' => self.emit_simple(TK::LParen, "(", start_line, start_col),
                ')' => self.emit_simple(TK::RParen, ")", start_line, start_col),
                '[' => self.emit_simple(TK::LBracket, "[", start_line, start_col),
                ']' => self.emit_simple(TK::RBracket, "]", start_line, start_col),
                ',' => self.emit_simple(TK::Comma, ",", start_line, start_col),
                '|' => self.emit_simple(TK::Pipe, "|", start_line, start_col),
                ':' => self.emit_simple(TK::Colon, ":", start_line, start_col),
                '%' => self.emit_simple(TK::Percent, "%", start_line, start_col),
                '?' => self.emit_simple(TK::Question, "?", start_line, start_col),
                '-' => self.emit_simple(TK::Minus, "-", start_line, start_col),
                '*' => self.emit_simple(TK::Star, "*", start_line, start_col),
                '/' => self.emit_simple(TK::Slash, "/", start_line, start_col),
                '+' => {
                    if self.cur_byte() == Some(b'=') {
                        self.advance_byte();
                        self.emit(TK::PlusEq, "+=".into(), start_line, start_col);
                    } else {
                        self.emit_simple(TK::Plus, "+", start_line, start_col);
                    }
                }
                '.' => {
                    if self.cur_byte() == Some(b'.') {
                        self.advance_byte();
                        self.emit(TK::DotDot, "..".into(), start_line, start_col);
                    } else {
                        self.emit_simple(TK::Dot, ".", start_line, start_col);
                    }
                }
                '=' => {
                    if self.cur_byte() == Some(b'=') {
                        self.advance_byte();
                        self.emit(TK::EqEq, "==".into(), start_line, start_col);
                    } else if self.cur_byte() == Some(b'>') {
                        self.advance_byte();
                        self.emit(TK::Arrow, "=>".into(), start_line, start_col);
                    } else {
                        self.emit_simple(TK::Eq, "=", start_line, start_col);
                    }
                }
                '!' => {
                    if self.cur_byte() == Some(b'=') {
                        self.advance_byte();
                        self.emit(TK::BangEq, "!=".into(), start_line, start_col);
                    }
                }
                '<' => {
                    if self.cur_byte() == Some(b'=') {
                        self.advance_byte();
                        self.emit(TK::LtEq, "<=".into(), start_line, start_col);
                    } else {
                        self.emit_simple(TK::Lt, "<", start_line, start_col);
                    }
                }
                '>' => {
                    if self.cur_byte() == Some(b'=') {
                        self.advance_byte();
                        self.emit(TK::GtEq, ">=".into(), start_line, start_col);
                    } else {
                        self.emit_simple(TK::Gt, ">", start_line, start_col);
                    }
                }
                _ => {}
            }
        }

        self.emit(TK::Eof, String::new(), self.line, self.col);
        self.tokens
    }
}

pub fn lex(src: &str) -> Vec<Token> {
    Lexer::new(src).run()
}
