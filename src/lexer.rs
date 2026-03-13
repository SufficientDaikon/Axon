// lexer.rs — Tokenizer for the Axon programming language
//
// Handles: keywords, types, operators, delimiters, literals (int, float,
// string, char), comments (// and /* */), attributes (@cpu, @gpu, @device),
// identifiers, and source location tracking.

use crate::error::CompileError;
use crate::span::Span;
use crate::token::{lookup_keyword, Token, TokenKind};

/// The Axon lexer. Produces a stream of tokens from source text.
pub struct Lexer {
    source: Vec<char>,
    file: String,
    pos: usize,
    line: usize,
    column: usize,
    pub errors: Vec<CompileError>,
}

impl Lexer {
    pub fn new(source: &str, file: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            file: file.to_string(),
            pos: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    // ── Helpers ──────────────────────────────────────────────

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn make_span(&self, start_line: usize, start_col: usize) -> Span {
        Span::new(&self.file, start_line, start_col, self.line, self.column)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn report_error(&mut self, code: &str, message: impl Into<String>, span: Span) {
        self.errors.push(CompileError::new(code, message, span));
    }

    // ── Main tokenize entry point ────────────────────────────

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                tokens.push(Token::new(
                    TokenKind::Eof,
                    self.make_span(self.line, self.column),
                ));
                break;
            }

            match self.next_token() {
                Some(tok) => tokens.push(tok),
                None => {
                    // Error already reported; skip the bad character
                    if !self.is_at_end() {
                        let sl = self.line;
                        let sc = self.column;
                        let bad = self.advance().unwrap();
                        self.report_error(
                            "E0001",
                            format!("unexpected character '{}'", bad),
                            self.make_span(sl, sc),
                        );
                    }
                }
            }
        }

        tokens
    }

    // ── Whitespace & comments ────────────────────────────────

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Skip single-line comments: //
            if self.peek() == Some('/') && self.peek_ahead(1) == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Skip multi-line comments: /* ... */
            if self.peek() == Some('/') && self.peek_ahead(1) == Some('*') {
                let sl = self.line;
                let sc = self.column;
                self.advance(); // /
                self.advance(); // *
                let mut depth = 1;
                while depth > 0 {
                    if self.is_at_end() {
                        self.report_error(
                            "E0002",
                            "unterminated block comment",
                            self.make_span(sl, sc),
                        );
                        return;
                    }
                    if self.peek() == Some('/') && self.peek_ahead(1) == Some('*') {
                        self.advance();
                        self.advance();
                        depth += 1;
                    } else if self.peek() == Some('*') && self.peek_ahead(1) == Some('/') {
                        self.advance();
                        self.advance();
                        depth -= 1;
                    } else {
                        self.advance();
                    }
                }
                continue;
            }

            break;
        }
    }

    // ── Token dispatch ───────────────────────────────────────

    fn next_token(&mut self) -> Option<Token> {
        let ch = self.peek()?;
        let sl = self.line;
        let sc = self.column;

        match ch {
            // ── String literal ───────────────────────────────
            '"' => Some(self.lex_string()),

            // ── Char literal ─────────────────────────────────
            '\'' => Some(self.lex_char()),

            // ── Number literals ──────────────────────────────
            '0'..='9' => Some(self.lex_number()),

            // ── Identifiers and keywords ─────────────────────
            'a'..='z' | 'A'..='Z' | '_' => Some(self.lex_identifier_or_keyword()),

            // ── @ (attribute or matrix multiply) ─────────────
            '@' => Some(self.lex_at()),

            // ── Two-char operators first, then single-char ───
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::EqEq, self.make_span(sl, sc)))
                } else if self.peek() == Some('>') {
                    self.advance();
                    Some(Token::new(TokenKind::FatArrow, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Eq, self.make_span(sl, sc)))
                }
            }

            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::NotEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Bang, self.make_span(sl, sc)))
                }
            }

            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::LtEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Lt, self.make_span(sl, sc)))
                }
            }

            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::GtEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Gt, self.make_span(sl, sc)))
                }
            }

            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    Some(Token::new(TokenKind::AndAnd, self.make_span(sl, sc)))
                } else if self.peek_is_keyword("mut") {
                    // Consume 'mut'
                    self.advance(); // m
                    self.advance(); // u
                    self.advance(); // t
                    Some(Token::new(TokenKind::AmpMut, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Amp, self.make_span(sl, sc)))
                }
            }

            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    Some(Token::new(TokenKind::OrOr, self.make_span(sl, sc)))
                } else if self.peek() == Some('>') {
                    self.advance();
                    Some(Token::new(TokenKind::Pipe, self.make_span(sl, sc)))
                } else {
                    // Bare `|` — used for closures; for now treat as unexpected
                    // We'll report an error
                    self.report_error(
                        "E0001",
                        "unexpected character '|' (did you mean '||'?)",
                        self.make_span(sl, sc),
                    );
                    None
                }
            }

            '+' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::PlusEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Plus, self.make_span(sl, sc)))
                }
            }

            '-' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::MinusEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Minus, self.make_span(sl, sc)))
                }
            }

            '*' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::StarEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Star, self.make_span(sl, sc)))
                }
            }

            '/' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(TokenKind::SlashEq, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Slash, self.make_span(sl, sc)))
                }
            }

            '%' => {
                self.advance();
                Some(Token::new(TokenKind::Percent, self.make_span(sl, sc)))
            }

            '?' => {
                self.advance();
                Some(Token::new(TokenKind::Question, self.make_span(sl, sc)))
            }

            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    Some(Token::new(TokenKind::DotDot, self.make_span(sl, sc)))
                } else {
                    Some(Token::new(TokenKind::Dot, self.make_span(sl, sc)))
                }
            }

            ':' => {
                self.advance();
                Some(Token::new(TokenKind::Colon, self.make_span(sl, sc)))
            }

            // ── Simple delimiters ────────────────────────────
            '(' => { self.advance(); Some(Token::new(TokenKind::LParen, self.make_span(sl, sc))) }
            ')' => { self.advance(); Some(Token::new(TokenKind::RParen, self.make_span(sl, sc))) }
            '{' => { self.advance(); Some(Token::new(TokenKind::LBrace, self.make_span(sl, sc))) }
            '}' => { self.advance(); Some(Token::new(TokenKind::RBrace, self.make_span(sl, sc))) }
            '[' => { self.advance(); Some(Token::new(TokenKind::LBracket, self.make_span(sl, sc))) }
            ']' => { self.advance(); Some(Token::new(TokenKind::RBracket, self.make_span(sl, sc))) }
            ',' => { self.advance(); Some(Token::new(TokenKind::Comma, self.make_span(sl, sc))) }
            ';' => { self.advance(); Some(Token::new(TokenKind::Semicolon, self.make_span(sl, sc))) }

            _ => None, // Unrecognized character — caller will report error
        }
    }

    // ── Identifier / Keyword lexing ──────────────────────────

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        let mut ident = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = lookup_keyword(&ident).unwrap_or(TokenKind::Identifier(ident));
        Token::new(kind, self.make_span(sl, sc))
    }

    // ── Number lexing ────────────────────────────────────────

    fn lex_number(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        let mut num_str = String::new();
        let mut is_float = false;

        // Check for hex/binary/octal prefix
        if self.peek() == Some('0') {
            match self.peek_ahead(1) {
                Some('x') | Some('X') => return self.lex_hex_number(),
                Some('b') | Some('B') => return self.lex_binary_number(),
                Some('o') | Some('O') => return self.lex_octal_number(),
                _ => {}
            }
        }

        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                if c != '_' {
                    num_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        if self.peek() == Some('.') && self.peek_ahead(1).is_some_and(|c| c.is_ascii_digit()) {
            is_float = true;
            num_str.push('.');
            self.advance(); // consume '.'
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        num_str.push(c);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if let Some('e') | Some('E') = self.peek() {
            is_float = true;
            num_str.push('e');
            self.advance();
            if let Some('+') | Some('-') = self.peek() {
                num_str.push(self.advance().unwrap());
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == '_' {
                    if c != '_' {
                        num_str.push(c);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let span = self.make_span(sl, sc);

        if is_float {
            match num_str.parse::<f64>() {
                Ok(val) => Token::new(TokenKind::FloatLiteral(val), span),
                Err(_) => {
                    self.report_error("E0003", format!("invalid float literal: {}", num_str), span.clone());
                    Token::new(TokenKind::FloatLiteral(0.0), span)
                }
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(val) => Token::new(TokenKind::IntLiteral(val), span),
                Err(_) => {
                    self.report_error("E0003", format!("invalid integer literal: {}", num_str), span.clone());
                    Token::new(TokenKind::IntLiteral(0), span)
                }
            }
        }
    }

    fn lex_hex_number(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // 0
        self.advance(); // x
        let mut hex_str = String::new();

        while let Some(c) = self.peek() {
            if c.is_ascii_hexdigit() || c == '_' {
                if c != '_' {
                    hex_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        let span = self.make_span(sl, sc);
        match i64::from_str_radix(&hex_str, 16) {
            Ok(val) => Token::new(TokenKind::IntLiteral(val), span),
            Err(_) => {
                self.report_error("E0003", format!("invalid hex literal: 0x{}", hex_str), span.clone());
                Token::new(TokenKind::IntLiteral(0), span)
            }
        }
    }

    fn lex_binary_number(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // 0
        self.advance(); // b
        let mut bin_str = String::new();

        while let Some(c) = self.peek() {
            if c == '0' || c == '1' || c == '_' {
                if c != '_' {
                    bin_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        let span = self.make_span(sl, sc);
        match i64::from_str_radix(&bin_str, 2) {
            Ok(val) => Token::new(TokenKind::IntLiteral(val), span),
            Err(_) => {
                self.report_error("E0003", format!("invalid binary literal: 0b{}", bin_str), span.clone());
                Token::new(TokenKind::IntLiteral(0), span)
            }
        }
    }

    fn lex_octal_number(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // 0
        self.advance(); // o
        let mut oct_str = String::new();

        while let Some(c) = self.peek() {
            if ('0'..='7').contains(&c) || c == '_' {
                if c != '_' {
                    oct_str.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        let span = self.make_span(sl, sc);
        match i64::from_str_radix(&oct_str, 8) {
            Ok(val) => Token::new(TokenKind::IntLiteral(val), span),
            Err(_) => {
                self.report_error("E0003", format!("invalid octal literal: 0o{}", oct_str), span.clone());
                Token::new(TokenKind::IntLiteral(0), span)
            }
        }
    }

    // ── String literal lexing ────────────────────────────────

    fn lex_string(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // opening "
        let mut value = String::new();

        loop {
            match self.peek() {
                None => {
                    self.report_error(
                        "E0004",
                        "unterminated string literal",
                        self.make_span(sl, sc),
                    );
                    break;
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => { value.push('\n'); self.advance(); }
                        Some('t') => { value.push('\t'); self.advance(); }
                        Some('r') => { value.push('\r'); self.advance(); }
                        Some('\\') => { value.push('\\'); self.advance(); }
                        Some('"') => { value.push('"'); self.advance(); }
                        Some('0') => { value.push('\0'); self.advance(); }
                        Some(c) => {
                            self.report_error(
                                "E0005",
                                format!("invalid escape sequence: \\{}", c),
                                self.make_span(self.line, self.column - 1),
                            );
                            self.advance();
                        }
                        None => {
                            self.report_error(
                                "E0004",
                                "unterminated string literal",
                                self.make_span(sl, sc),
                            );
                            break;
                        }
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
            }
        }

        Token::new(TokenKind::StringLiteral(value), self.make_span(sl, sc))
    }

    // ── Char literal lexing ──────────────────────────────────

    fn lex_char(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // opening '

        let ch = match self.peek() {
            Some('\\') => {
                self.advance();
                match self.peek() {
                    Some('n') => { self.advance(); '\n' }
                    Some('t') => { self.advance(); '\t' }
                    Some('r') => { self.advance(); '\r' }
                    Some('\\') => { self.advance(); '\\' }
                    Some('\'') => { self.advance(); '\'' }
                    Some('0') => { self.advance(); '\0' }
                    Some(c) => {
                        self.report_error(
                            "E0005",
                            format!("invalid escape sequence: \\{}", c),
                            self.make_span(sl, sc),
                        );
                        self.advance();
                        '\0'
                    }
                    None => {
                        self.report_error("E0006", "unterminated char literal", self.make_span(sl, sc));
                        return Token::new(TokenKind::CharLiteral('\0'), self.make_span(sl, sc));
                    }
                }
            }
            Some(c) => {
                self.advance();
                c
            }
            None => {
                self.report_error("E0006", "unterminated char literal", self.make_span(sl, sc));
                return Token::new(TokenKind::CharLiteral('\0'), self.make_span(sl, sc));
            }
        };

        if self.peek() == Some('\'') {
            self.advance();
        } else {
            self.report_error("E0006", "unterminated char literal (expected closing ')", self.make_span(sl, sc));
        }

        Token::new(TokenKind::CharLiteral(ch), self.make_span(sl, sc))
    }

    // ── @ attribute lexing ───────────────────────────────────

    fn lex_at(&mut self) -> Token {
        let sl = self.line;
        let sc = self.column;
        self.advance(); // @

        // Check if followed by an identifier
        if let Some(c) = self.peek() {
            if c.is_alphabetic() || c == '_' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                let span = self.make_span(sl, sc);
                return match ident.as_str() {
                    "cpu" => Token::new(TokenKind::AtCpu, span),
                    "gpu" => Token::new(TokenKind::AtGpu, span),
                    "device" => Token::new(TokenKind::AtDevice, span),
                    _ => {
                        self.report_error(
                            "E0007",
                            format!("unknown attribute: @{}", ident),
                            span.clone(),
                        );
                        Token::new(TokenKind::At, span)
                    }
                };
            }
        }

        // Plain @ operator (matrix multiply)
        Token::new(TokenKind::At, self.make_span(sl, sc))
    }

    // ── Helper: check if next chars spell a keyword ──────────

    fn peek_is_keyword(&self, kw: &str) -> bool {
        let chars: Vec<char> = kw.chars().collect();
        for (i, &expected) in chars.iter().enumerate() {
            match self.source.get(self.pos + i) {
                Some(&c) if c == expected => continue,
                _ => return false,
            }
        }
        // Make sure the keyword isn't part of a longer identifier
        match self.source.get(self.pos + chars.len()) {
            Some(&c) if c.is_alphanumeric() || c == '_' => false,
            _ => true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(src, "test.axon");
        lexer.tokenize()
    }

    fn kinds(tokens: &[Token]) -> Vec<&TokenKind> {
        tokens.iter().map(|t| &t.kind).collect()
    }

    // ── Keywords ─────────────────────────────────────────────

    #[test]
    fn test_keywords() {
        let tokens = lex("fn mut val var model enum extend trait if else while for in match return pub use mod unsafe self true false as type");
        let expected = vec![
            TokenKind::Fn, TokenKind::Mut, TokenKind::Val, TokenKind::Var,
            TokenKind::Model, TokenKind::Enum, TokenKind::Extend, TokenKind::Trait,
            TokenKind::If, TokenKind::Else, TokenKind::While, TokenKind::For,
            TokenKind::In, TokenKind::Match, TokenKind::Return, TokenKind::Pub,
            TokenKind::Use, TokenKind::Mod, TokenKind::Unsafe, TokenKind::SelfValue,
            TokenKind::True, TokenKind::False, TokenKind::As, TokenKind::Type,
            TokenKind::Eof,
        ];
        let actual: Vec<&TokenKind> = kinds(&tokens);
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(actual[i], exp, "Mismatch at index {}", i);
        }
    }

    // ── Built-in types ───────────────────────────────────────

    #[test]
    fn test_builtin_types() {
        let tokens = lex("Int8 Int16 Int32 Int64 UInt8 UInt16 UInt32 UInt64 Float16 Float32 Float64 Bool Char String Tensor Vec HashMap HashSet Option Result Mutex RwLock Channel Arc");
        let expected = vec![
            TokenKind::Int8, TokenKind::Int16, TokenKind::Int32, TokenKind::Int64,
            TokenKind::UInt8, TokenKind::UInt16, TokenKind::UInt32, TokenKind::UInt64,
            TokenKind::Float16, TokenKind::Float32, TokenKind::Float64,
            TokenKind::BoolType, TokenKind::CharType, TokenKind::StringType,
            TokenKind::TensorType, TokenKind::VecType, TokenKind::HashMapType,
            TokenKind::HashSetType, TokenKind::OptionType, TokenKind::ResultType,
            TokenKind::MutexType, TokenKind::RwLockType, TokenKind::ChannelType,
            TokenKind::ArcType, TokenKind::Eof,
        ];
        let actual: Vec<&TokenKind> = kinds(&tokens);
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(actual[i], exp, "Mismatch at index {}", i);
        }
    }

    // ── Operators ────────────────────────────────────────────

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / @ % == != < > <= >= && || ! & ? => |> . .. = += -= *= /=");
        let expected = vec![
            TokenKind::Plus, TokenKind::Minus, TokenKind::Star, TokenKind::Slash,
            TokenKind::At, TokenKind::Percent, TokenKind::EqEq, TokenKind::NotEq,
            TokenKind::Lt, TokenKind::Gt, TokenKind::LtEq, TokenKind::GtEq,
            TokenKind::AndAnd, TokenKind::OrOr, TokenKind::Bang, TokenKind::Amp,
            TokenKind::Question, TokenKind::FatArrow,
            TokenKind::Pipe, TokenKind::Dot, TokenKind::DotDot,
            TokenKind::Eq, TokenKind::PlusEq, TokenKind::MinusEq,
            TokenKind::StarEq, TokenKind::SlashEq, TokenKind::Eof,
        ];
        let actual: Vec<&TokenKind> = kinds(&tokens);
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(actual[i], exp, "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_amp_mut() {
        let tokens = lex("&mut self");
        assert_eq!(tokens[0].kind, TokenKind::AmpMut);
        assert_eq!(tokens[1].kind, TokenKind::SelfValue);
    }

    // ── Delimiters ───────────────────────────────────────────

    #[test]
    fn test_delimiters() {
        let tokens = lex("( ) { } [ ] , ; :");
        let expected = vec![
            TokenKind::LParen, TokenKind::RParen, TokenKind::LBrace,
            TokenKind::RBrace, TokenKind::LBracket, TokenKind::RBracket,
            TokenKind::Comma, TokenKind::Semicolon, TokenKind::Colon,
            TokenKind::Eof,
        ];
        let actual: Vec<&TokenKind> = kinds(&tokens);
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(actual[i], exp, "Mismatch at index {}", i);
        }
    }

    // ── Integer literals ─────────────────────────────────────

    #[test]
    fn test_integer_literals() {
        let tokens = lex("42 0 1000 0xFF 0b1010 0o77");
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(0));
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(1000));
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral(0xFF));
        assert_eq!(tokens[4].kind, TokenKind::IntLiteral(0b1010));
        assert_eq!(tokens[5].kind, TokenKind::IntLiteral(0o77));
    }

    #[test]
    fn test_underscored_numbers() {
        let tokens = lex("1_000_000 0xFF_FF");
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(1_000_000));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(0xFFFF));
    }

    // ── Float literals ───────────────────────────────────────

    #[test]
    fn test_float_literals() {
        let tokens = lex("3.14 1.0e10 2.5e-3 0.0");
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral(3.14));
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral(1.0e10));
        assert_eq!(tokens[2].kind, TokenKind::FloatLiteral(2.5e-3));
        assert_eq!(tokens[3].kind, TokenKind::FloatLiteral(0.0));
    }

    // ── String literals ──────────────────────────────────────

    #[test]
    fn test_string_literal() {
        let tokens = lex(r#""Hello, Axon!""#);
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("Hello, Axon!".to_string()));
    }

    #[test]
    fn test_string_escapes() {
        let tokens = lex(r#""hello\nworld\t!""#);
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello\nworld\t!".to_string()));
    }

    // ── Char literals ────────────────────────────────────────

    #[test]
    fn test_char_literal() {
        let tokens = lex("'a' 'Z' '\\n'");
        assert_eq!(tokens[0].kind, TokenKind::CharLiteral('a'));
        assert_eq!(tokens[1].kind, TokenKind::CharLiteral('Z'));
        assert_eq!(tokens[2].kind, TokenKind::CharLiteral('\n'));
    }

    // ── Boolean literals ─────────────────────────────────────

    #[test]
    fn test_booleans() {
        let tokens = lex("true false");
        assert_eq!(tokens[0].kind, TokenKind::True);
        assert_eq!(tokens[1].kind, TokenKind::False);
    }

    // ── Comments ─────────────────────────────────────────────

    #[test]
    fn test_single_line_comment() {
        let tokens = lex("42 // this is a comment\n43");
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(43));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn test_multi_line_comment() {
        let tokens = lex("42 /* this\nis\na comment */ 43");
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(43));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn test_nested_comments() {
        let tokens = lex("1 /* outer /* inner */ still comment */ 2");
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(1));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(2));
    }

    // ── Attributes ───────────────────────────────────────────

    #[test]
    fn test_attributes() {
        let tokens = lex("@cpu @gpu @device");
        assert_eq!(tokens[0].kind, TokenKind::AtCpu);
        assert_eq!(tokens[1].kind, TokenKind::AtGpu);
        assert_eq!(tokens[2].kind, TokenKind::AtDevice);
    }

    #[test]
    fn test_at_operator() {
        // Bare @ without identifier is matrix multiply
        let tokens = lex("A @ B");
        assert_eq!(tokens[0].kind, TokenKind::Identifier("A".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::At);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("B".to_string()));
    }

    // ── Identifiers ──────────────────────────────────────────

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar_baz _private x123");
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar_baz".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("_private".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Identifier("x123".to_string()));
    }

    // ── Source location tracking ─────────────────────────────

    #[test]
    fn test_source_location() {
        let tokens = lex("let x = 10;");
        assert_eq!(tokens[0].span.start.line, 1);
        assert_eq!(tokens[0].span.start.column, 1);
        // `x` starts at column 5
        assert_eq!(tokens[1].span.start.line, 1);
        assert_eq!(tokens[1].span.start.column, 5);
    }

    #[test]
    fn test_multiline_location() {
        let tokens = lex("let\nx\n=\n10");
        assert_eq!(tokens[0].span.start.line, 1); // let
        assert_eq!(tokens[1].span.start.line, 2); // x
        assert_eq!(tokens[2].span.start.line, 3); // =
        assert_eq!(tokens[3].span.start.line, 4); // 10
    }

    // ── Error recovery ───────────────────────────────────────

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""hello"#, "test.axon");
        let _tokens = lexer.tokenize();
        assert!(!lexer.errors.is_empty());
        assert_eq!(lexer.errors[0].error_code, "E0004");
    }

    #[test]
    fn test_unterminated_block_comment() {
        let mut lexer = Lexer::new("/* unclosed", "test.axon");
        let _tokens = lexer.tokenize();
        assert!(!lexer.errors.is_empty());
        assert_eq!(lexer.errors[0].error_code, "E0002");
    }

    // ── Complex token sequences ──────────────────────────────

    #[test]
    fn test_fn_declaration_tokens() {
        let tokens = lex("fn main() { println(\"Hello\"); }");
        let expected = vec![
            TokenKind::Fn,
            TokenKind::Identifier("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Identifier("println".to_string()),
            TokenKind::LParen,
            TokenKind::StringLiteral("Hello".to_string()),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];
        let actual: Vec<&TokenKind> = kinds(&tokens);
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(actual[i], exp, "Mismatch at index {}", i);
        }
    }

    #[test]
    fn test_tensor_type_tokens() {
        let tokens = lex("Tensor<Float32, [128, 256]>");
        let expected_kinds = vec![
            &TokenKind::TensorType,
            &TokenKind::Lt,
            &TokenKind::Float32,
            &TokenKind::Comma,
            &TokenKind::LBracket,
            &TokenKind::IntLiteral(128),
            &TokenKind::Comma,
            &TokenKind::IntLiteral(256),
            &TokenKind::RBracket,
            &TokenKind::Gt,
            &TokenKind::Eof,
        ];
        assert_eq!(kinds(&tokens), expected_kinds);
    }

    #[test]
    fn test_fat_arrow_operator() {
        let tokens = lex("=>");
        assert_eq!(tokens[0].kind, TokenKind::FatArrow);
    }

    #[test]
    fn test_compound_assignment() {
        let tokens = lex("x += 1; y -= 2; z *= 3; w /= 4;");
        assert_eq!(tokens[1].kind, TokenKind::PlusEq);
        assert_eq!(tokens[5].kind, TokenKind::MinusEq);
        assert_eq!(tokens[9].kind, TokenKind::StarEq);
        assert_eq!(tokens[13].kind, TokenKind::SlashEq);
    }
}
