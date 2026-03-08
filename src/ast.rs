// ast.rs — Abstract Syntax Tree node definitions for Axon

use serde::Serialize;
use crate::span::Span;

// ═══════════════════════════════════════════════════════════════
// Top-Level Program
// ═══════════════════════════════════════════════════════════════

/// The root of an Axon program: a sequence of top-level items.
#[derive(Debug, Clone, Serialize)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// A top-level item in a program or module.
#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Debug, Clone, Serialize)]
pub enum Attribute {
    Cpu,
    Gpu,
    Device(Expr),
}

#[derive(Debug, Clone, Serialize)]
pub enum ItemKind {
    Function(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(ImplBlock),
    Trait(TraitDecl),
    TypeAlias(TypeAliasDecl),
    Module(ModuleDecl),
    Use(UseDecl),
}

// ═══════════════════════════════════════════════════════════════
// Declarations
// ═══════════════════════════════════════════════════════════════

/// Function declaration: `fn name<T: Bound>(params) -> ReturnType { body }`
#[derive(Debug, Clone, Serialize)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<FnParam>,
    pub return_type: Option<TypeExpr>,
    pub body: Option<Block>,
    pub span: Span,
}

/// A generic type parameter: `T: Bound1 + Bound2`
#[derive(Debug, Clone, Serialize)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeExpr>,
    pub span: Span,
}

/// A function parameter: `name: Type` or `&self` / `&mut self`
#[derive(Debug, Clone, Serialize)]
pub struct FnParam {
    pub kind: FnParamKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum FnParamKind {
    /// A normal typed parameter: `name: Type`
    Typed {
        name: String,
        ty: TypeExpr,
        default: Option<Expr>,
    },
    /// `self` (takes ownership)
    SelfOwned,
    /// `&self` (immutable borrow)
    SelfRef,
    /// `&mut self` (mutable borrow)
    SelfMutRef,
}

/// Struct declaration: `struct Name<T> { field: Type, ... }`
#[derive(Debug, Clone, Serialize)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructField {
    pub name: String,
    pub ty: TypeExpr,
    pub visibility: Visibility,
    pub span: Span,
}

/// Enum declaration: `enum Name<T> { Variant1, Variant2(Type), ... }`
#[derive(Debug, Clone, Serialize)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: EnumVariantKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum EnumVariantKind {
    /// Unit variant: `Variant`
    Unit,
    /// Tuple variant: `Variant(Type1, Type2)`
    Tuple(Vec<TypeExpr>),
    /// Struct variant: `Variant { field: Type }`
    Struct(Vec<StructField>),
}

/// Impl block: `impl Type { ... }` or `impl Trait for Type { ... }`
#[derive(Debug, Clone, Serialize)]
pub struct ImplBlock {
    pub type_name: TypeExpr,
    pub trait_name: Option<TypeExpr>,
    pub generics: Vec<GenericParam>,
    pub items: Vec<Item>,
    pub span: Span,
}

/// Trait declaration: `trait Name<T>: SuperTrait { ... }`
#[derive(Debug, Clone, Serialize)]
pub struct TraitDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub supertraits: Vec<TypeExpr>,
    pub items: Vec<Item>,
    pub span: Span,
}

/// Type alias: `type Name<T> = Type;`
#[derive(Debug, Clone, Serialize)]
pub struct TypeAliasDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Module declaration: `mod name { ... }` or `mod name;`
#[derive(Debug, Clone, Serialize)]
pub struct ModuleDecl {
    pub name: String,
    pub items: Option<Vec<Item>>,
    pub span: Span,
}

/// Use declaration: `use path::to::item;` or `use path::to::{a, b};`
#[derive(Debug, Clone, Serialize)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum StmtKind {
    /// `let [mut] name [: Type] = expr;`
    Let {
        name: Pattern,
        mutable: bool,
        ty: Option<TypeExpr>,
        initializer: Option<Expr>,
    },
    /// An expression used as a statement
    Expr(Expr),
    /// `return [expr];`
    Return(Option<Expr>),
    /// `while condition { body }`
    While {
        condition: Expr,
        body: Block,
    },
    /// `for pattern in iterator { body }`
    For {
        pattern: Pattern,
        iterator: Expr,
        body: Block,
    },
    /// An item declared inside a block (e.g., a nested function)
    Item(Item),
}

/// A block: `{ stmts... [tail_expr] }`
#[derive(Debug, Clone, Serialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// Optional trailing expression (the block's value)
    pub tail_expr: Option<Box<Expr>>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Expressions
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum ExprKind {
    /// A literal value: `42`, `3.14`, `"hello"`, `true`, `'a'`
    Literal(Literal),

    /// An identifier: `x`, `foo`
    Identifier(String),

    /// A path expression: `Foo::bar`, `std::io::Error`
    Path(Vec<String>),

    /// Binary operation: `a + b`, `a @ b`, etc.
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    /// Unary operation: `!x`, `-x`
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    /// Function call: `foo(a, b, c)`
    FnCall {
        function: Box<Expr>,
        args: Vec<Expr>,
    },

    /// Method call: `obj.method(a, b)`
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// Field access: `obj.field`
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// Index access: `arr[i]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// Slice: `arr[start..end]`
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },

    /// If-else expression: `if cond { a } else { b }`
    IfElse {
        condition: Box<Expr>,
        then_block: Block,
        else_block: Option<ElseClause>,
    },

    /// Match expression: `match expr { arms }`
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Block expression: `{ stmts...; value }`
    Block(Block),

    /// Reference: `&expr` or `&mut expr`
    Reference {
        mutable: bool,
        expr: Box<Expr>,
    },

    /// Type cast: `expr as Type`
    TypeCast {
        expr: Box<Expr>,
        target_type: TypeExpr,
    },

    /// Error propagation: `expr?`
    ErrorPropagation(Box<Expr>),

    /// Range: `start..end`
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },

    /// Assignment: `lhs = rhs` or `lhs += rhs`
    Assignment {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
    },

    /// Tuple: `(a, b, c)`
    Tuple(Vec<Expr>),

    /// Struct literal: `Name { field: value, ... }`
    StructLiteral {
        name: Vec<String>,
        fields: Vec<StructLiteralField>,
    },

    /// Closure: `|params| body` or `|params| -> Type { body }`
    Closure {
        params: Vec<ClosureParam>,
        return_type: Option<TypeExpr>,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ClosureParam {
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructLiteralField {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum ElseClause {
    ElseBlock(Block),
    ElseIf(Box<Expr>), // The Expr is an IfElse
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

// ── Operators ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinOp {
    Add,     // +
    Sub,     // -
    Mul,     // *
    Div,     // /
    Mod,     // %
    MatMul,  // @ (FR-002)
    Eq,      // ==
    NotEq,   // !=
    Lt,      // <
    Gt,      // >
    LtEq,    // <=
    GtEq,    // >=
    And,     // &&
    Or,      // ||
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // !
    Ref,    // &
    MutRef, // &mut
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AssignOp {
    Assign,   // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
}

// ── Literals ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
}

// ═══════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct TypeExpr {
    pub kind: TypeExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypeExprKind {
    /// A simple named type: `Int32`, `Bool`, `MyStruct`
    Named(String),

    /// A path type: `std::io::Error`
    Path(Vec<String>),

    /// A generic type: `Vec<Int32>`, `HashMap<String, Int32>`
    Generic {
        name: String,
        args: Vec<TypeArg>,
    },

    /// Tensor type: `Tensor<Float32, [128, 256]>` (FR-011)
    Tensor {
        dtype: Box<TypeExpr>,
        shape: Vec<ShapeDim>,
    },

    /// Reference type: `&T` or `&mut T` (FR-015)
    Reference {
        mutable: bool,
        inner: Box<TypeExpr>,
    },

    /// Function type: `fn(A, B) -> C`
    Function {
        params: Vec<TypeExpr>,
        return_type: Box<TypeExpr>,
    },

    /// Tuple type: `(A, B, C)`
    Tuple(Vec<TypeExpr>),

    /// Array type: `[T; N]`
    Array {
        element: Box<TypeExpr>,
        size: usize,
    },

    /// Inferred type (placeholder for when no annotation is given)
    Inferred,
}

/// A type argument — can be a type or a shape literal.
#[derive(Debug, Clone, Serialize)]
pub enum TypeArg {
    Type(TypeExpr),
    Shape(Vec<ShapeDim>),
}

/// A single dimension in a tensor shape.
#[derive(Debug, Clone, Serialize)]
pub enum ShapeDim {
    /// A known constant dimension: `128`
    Constant(i64),
    /// A dynamic dimension: `?` (FR-014)
    Dynamic,
    /// A named generic dimension: `N`
    Named(String),
}

// ═══════════════════════════════════════════════════════════════
// Patterns
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum PatternKind {
    /// Identifier pattern: `x`, `name`
    Identifier(String),

    /// Literal pattern: `42`, `"hello"`, `true`
    Literal(Literal),

    /// Wildcard pattern: `_`
    Wildcard,

    /// Tuple pattern: `(a, b)`
    Tuple(Vec<Pattern>),

    /// Struct pattern: `Point { x, y }`
    Struct {
        name: Vec<String>,
        fields: Vec<FieldPattern>,
    },

    /// Enum variant pattern: `Some(x)`, `Ok(val)`
    EnumVariant {
        path: Vec<String>,
        fields: Vec<Pattern>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Option<Pattern>,
    pub span: Span,
}
