use crate::lexer::{TK, Token};
use tower_lsp_server::ls_types::{Position, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstelleType {
    Str,
    Num,
    Bool,
    List,
    Map,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub typ: EstelleType,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub alias: String,
    pub name_span: Range,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub name_span: Range,
    pub params: Vec<Param>,
    pub return_type: Option<EstelleType>,
    pub is_pub: bool,
    pub body_span: Range,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub range: Range,
}

pub struct DeclExtractor<'a> {
    tokens: &'a [Token],
    pos: usize,
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionDecl>,
    pub errors: Vec<ParseError>,
}

impl<'a> DeclExtractor<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        DeclExtractor {
            tokens,
            pos: 0,
            imports: Vec::new(),
            functions: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek(&self) -> TK {
        self.tokens[self.pos].kind.clone()
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn eat(&mut self, t: TK) -> bool {
        if self.peek() == t {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: TK, msg: &str) -> Option<Token> {
        if self.peek() == t {
            Some(self.advance().clone())
        } else {
            let tok = self.cur().clone();
            self.errors.push(ParseError {
                message: msg.to_string(),
                range: token_range(&tok),
            });
            Some(tok)
        }
    }

    fn is_name_token(k: &TK) -> bool {
        matches!(
            k,
            TK::Ident | TK::StrType | TK::NumType | TK::BoolType | TK::ListType | TK::MapType
        )
    }

    fn expect_name(&mut self, msg: &str) -> Option<Token> {
        if Self::is_name_token(&self.peek()) {
            Some(self.advance().clone())
        } else {
            self.expect(TK::Ident, msg)
        }
    }

    fn parse_type(&mut self) -> Option<(EstelleType, bool)> {
        let typ = match self.peek() {
            TK::StrType => EstelleType::Str,
            TK::NumType => EstelleType::Num,
            TK::BoolType => EstelleType::Bool,
            TK::ListType => EstelleType::List,
            TK::MapType => EstelleType::Map,
            _ => return None,
        };
        self.advance();
        let nullable = self.eat(TK::Question);
        Some((typ, nullable))
    }

    fn token_to_range(tok: &Token, len: usize) -> Range {
        let end_col = tok.col + len as u32;
        Range {
            start: Position {
                line: tok.line,
                character: tok.col,
            },
            end: Position {
                line: tok.line,
                character: end_col,
            },
        }
    }

    pub fn run(&mut self) {
        while self.peek() != TK::Eof {
            match self.peek() {
                TK::Import => {
                    self.parse_import();
                }
                TK::Pub | TK::Fnc => {
                    self.parse_fnc(0);
                }
                _ => {
                    let tok = self.cur();
                    self.errors.push(ParseError {
                        message: format!("Unexpected token \"{}\" at top level", tok.value),
                        range: token_range(tok),
                    });
                    self.advance();
                }
            }
        }
    }

    fn parse_import(&mut self) {
        self.advance(); // import
        let path_tok = match self.expect(TK::String, "Expected module path string after \"import\"")
        {
            Some(t) => t,
            None => return,
        };
        let path = path_tok.value.clone();

        let (alias, alias_span) = if self.eat(TK::As) {
            if let Some(alias_tok) = self.expect_name("Expected alias after \"as\"") {
                let alias = alias_tok.value.clone();
                let span = Self::token_to_range(&alias_tok, alias.len());
                (alias, span)
            } else {
                let alias = path.split(':').next_back().unwrap_or(&path).to_string();
                (alias.clone(), token_range(&path_tok))
            }
        } else {
            let alias = path.split(':').next_back().unwrap_or(&path).to_string();
            (alias.clone(), token_range(&path_tok))
        };

        self.imports.push(Import {
            path,
            alias,
            name_span: alias_span,
        });
    }

    fn parse_fnc(&mut self, depth: usize) {
        let mut is_pub = false;
        if self.peek() == TK::Pub {
            self.advance();
            if depth > 0 {
                // nested pub is invalid but we still want other decls soo
                let tok = self.cur();
                self.errors.push(ParseError {
                    message: "\"pub fnc\" is not allowed inside another function".to_string(),
                    range: token_range(tok),
                });
            } else {
                is_pub = true;
            }
        }

        if !self.eat(TK::Fnc) {
            self.expect(TK::Fnc, "Expected \"fnc\"");
            return;
        }

        let name_tok = match self.expect_name("Expected function name") {
            Some(t) => t,
            None => return,
        };
        let name = name_tok.value.clone();
        let name_span = Self::token_to_range(&name_tok, name.len());

        let mut params = Vec::new();
        if self.eat(TK::LParen) {
            while self.peek() != TK::RParen && self.peek() != TK::Eof {
                let pname_tok = match self.expect_name("Expected parameter name") {
                    Some(t) => t,
                    None => break,
                };
                let pname = pname_tok.value.clone();

                let ptype = self.parse_type();
                if ptype.is_none() {
                    self.errors.push(ParseError {
                        message: format!("Expected type for parameter \"{}\"", pname),
                        range: token_range(&pname_tok),
                    });
                }
                let (typ, nullable) = ptype.unwrap_or((EstelleType::Str, false));
                params.push(Param {
                    name: pname,
                    typ,
                    nullable,
                });

                if !self.eat(TK::Comma) {
                    break;
                }
            }
            self.expect(TK::RParen, "Expected \")\"");
        }

        let return_type = self.parse_type().map(|(t, _)| t);

        let body_start_tok = self.cur().clone();
        if !self.eat(TK::LBrace) {
            self.expect(TK::LBrace, "Expected \"{\"");
            return;
        }

        let body_start = Position {
            line: body_start_tok.line,
            character: body_start_tok.col,
        };

        self.skip_block(depth + 1);

        let mut body_end = Position {
            line: self.cur().line,
            character: self.cur().col,
        };
        if self.peek() == TK::RBrace {
            body_end.character += 1;
            self.advance();
        } else {
            self.expect(TK::RBrace, "Expected \"}\"");
        }

        let body_span = Range {
            start: body_start,
            end: body_end,
        };

        self.functions.push(FunctionDecl {
            name,
            name_span,
            params,
            return_type,
            is_pub,
            body_span,
        });
    }

    fn skip_block(&mut self, _depth: usize) {
        let mut brace_depth = 1usize;
        while self.peek() != TK::Eof && brace_depth > 0 {
            let tok_kind = self.peek();
            if tok_kind == TK::LBrace {
                brace_depth += 1;
                self.advance();
                continue;
            }
            if tok_kind == TK::RBrace {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return;
                }
                self.advance();
                continue;
            }
            // nested fnc at block level
            if brace_depth == 1 && (tok_kind == TK::Fnc || tok_kind == TK::Pub) {
                self.parse_fnc(1);
                continue;
            }
            if tok_kind == TK::LuaBlock || tok_kind == TK::OutputBlock {
                self.advance();
                continue;
            }
            self.advance();
        }
    }
}

fn token_range(tok: &Token) -> Range {
    let len = tok.value.len().max(1);
    Range {
        start: Position {
            line: tok.line,
            character: tok.col,
        },
        end: Position {
            line: tok.line,
            character: tok.col + len as u32,
        },
    }
}
