// token.rs — Token types for the Axon lexer

use serde::Serialize;
use crate::span::Span;

/// All token types in the Axon language.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenKind {
    // ── Keywords ──────────────────────────────────────────────
    Fn,
    Mut,
    Val,       // val (immutable binding)
    Var,       // var (mutable binding)
    Enum,
    Extend,    // extend (implementation block)
    Model,     // model (struct declaration)
    Trait,
    If,
    Else,
    While,
    For,
    In,
    Match,
    Return,
    Pub,
    Use,
    Mod,
    Unsafe,
    SelfValue, // `self`
    True,
    False,
    As,
    Type,

    // ── Built-in Type Names ──────────────────────────────────
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float16,
    Float32,
    Float64,
    BoolType,
    CharType,
    StringType,
    TensorType,
    VecType,
    HashMapType,
    HashSetType,
    OptionType,
    ResultType,
    MutexType,
    RwLockType,
    ChannelType,
    ArcType,

    // ── Operators ────────────────────────────────────────────
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    At,         // @ (matrix multiply)
    Percent,    // %
    EqEq,      // ==
    NotEq,     // !=
    Lt,        // <
    Gt,        // >
    LtEq,      // <=
    GtEq,      // >=
    AndAnd,    // &&
    OrOr,      // ||
    Bang,      // !
    Amp,       // &
    AmpMut,    // &mut
    Question,  // ?
    FatArrow,  // =>
    Pipe,      // |>
    Dot,       // .
    DotDot,    // ..
    Eq,        // =
    PlusEq,    // +=
    MinusEq,   // -=
    StarEq,    // *=
    SlashEq,   // /=

    // ── Delimiters ───────────────────────────────────────────
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Semicolon, // ;
    Colon,     // :

    // ── Literals ─────────────────────────────────────────────
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),

    // ── Attributes ───────────────────────────────────────────
    AtCpu,           // @cpu
    AtGpu,           // @gpu
    AtDevice,        // @device (followed by (...))

    // ── Identifiers ──────────────────────────────────────────
    Identifier(String),

    // ── Special ──────────────────────────────────────────────
    Eof,
}

impl TokenKind {
    /// Returns a human-readable name for this token kind.
    pub fn name(&self) -> &str {
        match self {
            TokenKind::Fn => "fn",
            TokenKind::Mut => "mut",
            TokenKind::Val => "val",
            TokenKind::Var => "var",
            TokenKind::Enum => "enum",
            TokenKind::Extend => "extend",
            TokenKind::Model => "model",
            TokenKind::Trait => "trait",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Match => "match",
            TokenKind::Return => "return",
            TokenKind::Pub => "pub",
            TokenKind::Use => "use",
            TokenKind::Mod => "mod",
            TokenKind::Unsafe => "unsafe",
            TokenKind::SelfValue => "self",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::As => "as",
            TokenKind::Type => "type",
            TokenKind::Int8 => "Int8",
            TokenKind::Int16 => "Int16",
            TokenKind::Int32 => "Int32",
            TokenKind::Int64 => "Int64",
            TokenKind::UInt8 => "UInt8",
            TokenKind::UInt16 => "UInt16",
            TokenKind::UInt32 => "UInt32",
            TokenKind::UInt64 => "UInt64",
            TokenKind::Float16 => "Float16",
            TokenKind::Float32 => "Float32",
            TokenKind::Float64 => "Float64",
            TokenKind::BoolType => "Bool",
            TokenKind::CharType => "Char",
            TokenKind::StringType => "String",
            TokenKind::TensorType => "Tensor",
            TokenKind::VecType => "Vec",
            TokenKind::HashMapType => "HashMap",
            TokenKind::HashSetType => "HashSet",
            TokenKind::OptionType => "Option",
            TokenKind::ResultType => "Result",
            TokenKind::MutexType => "Mutex",
            TokenKind::RwLockType => "RwLock",
            TokenKind::ChannelType => "Channel",
            TokenKind::ArcType => "Arc",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::At => "@",
            TokenKind::Percent => "%",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::LtEq => "<=",
            TokenKind::GtEq => ">=",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::Bang => "!",
            TokenKind::Amp => "&",
            TokenKind::AmpMut => "&mut",
            TokenKind::Question => "?",
            TokenKind::FatArrow => "=>",
            TokenKind::Pipe => "|>",
            TokenKind::Dot => ".",
            TokenKind::DotDot => "..",
            TokenKind::Eq => "=",
            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::StarEq => "*=",
            TokenKind::SlashEq => "/=",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::IntLiteral(_) => "integer literal",
            TokenKind::FloatLiteral(_) => "float literal",
            TokenKind::StringLiteral(_) => "string literal",
            TokenKind::CharLiteral(_) => "char literal",
            TokenKind::AtCpu => "@cpu",
            TokenKind::AtGpu => "@gpu",
            TokenKind::AtDevice => "@device",
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Eof => "end of file",
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::FloatLiteral(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::CharLiteral(c) => write!(f, "'{}'", c),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            other => write!(f, "{}", other.name()),
        }
    }
}

/// A token with its kind and source span.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}

/// Maps identifier strings to keyword token kinds.
pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        "fn" => Some(TokenKind::Fn),
        "mut" => Some(TokenKind::Mut),
        "val" => Some(TokenKind::Val),
        "var" => Some(TokenKind::Var),
        "model" => Some(TokenKind::Model),
        "enum" => Some(TokenKind::Enum),
        "extend" => Some(TokenKind::Extend),
        "trait" => Some(TokenKind::Trait),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "while" => Some(TokenKind::While),
        "for" => Some(TokenKind::For),
        "in" => Some(TokenKind::In),
        "match" => Some(TokenKind::Match),
        "return" => Some(TokenKind::Return),
        "pub" => Some(TokenKind::Pub),
        "use" => Some(TokenKind::Use),
        "mod" => Some(TokenKind::Mod),
        "unsafe" => Some(TokenKind::Unsafe),
        "self" => Some(TokenKind::SelfValue),
        "true" => Some(TokenKind::True),
        "false" => Some(TokenKind::False),
        "as" => Some(TokenKind::As),
        "type" => Some(TokenKind::Type),
        // Built-in types
        "Int8" => Some(TokenKind::Int8),
        "Int16" => Some(TokenKind::Int16),
        "Int32" => Some(TokenKind::Int32),
        "Int64" => Some(TokenKind::Int64),
        "UInt8" => Some(TokenKind::UInt8),
        "UInt16" => Some(TokenKind::UInt16),
        "UInt32" => Some(TokenKind::UInt32),
        "UInt64" => Some(TokenKind::UInt64),
        "Float16" => Some(TokenKind::Float16),
        "Float32" => Some(TokenKind::Float32),
        "Float64" => Some(TokenKind::Float64),
        "Bool" => Some(TokenKind::BoolType),
        "Char" => Some(TokenKind::CharType),
        "String" => Some(TokenKind::StringType),
        "Tensor" => Some(TokenKind::TensorType),
        "Vec" => Some(TokenKind::VecType),
        "HashMap" => Some(TokenKind::HashMapType),
        "HashSet" => Some(TokenKind::HashSetType),
        "Option" => Some(TokenKind::OptionType),
        "Result" => Some(TokenKind::ResultType),
        "Mutex" => Some(TokenKind::MutexType),
        "RwLock" => Some(TokenKind::RwLockType),
        "Channel" => Some(TokenKind::ChannelType),
        "Arc" => Some(TokenKind::ArcType),
        _ => None,
    }
}
