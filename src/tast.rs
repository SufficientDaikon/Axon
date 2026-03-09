// tast.rs — Typed Abstract Syntax Tree (Phase 3f output)
//
// Mirrors src/ast.rs but every node carries a resolved TypeId.
// This is the output of Phase 3 and the input to Phase 4 codegen.

use serde::Serialize;
use crate::span::Span;
use crate::types::TypeId;
use crate::ast::{Visibility, Attribute, BinOp, UnaryOp, AssignOp, Literal};

// ═══════════════════════════════════════════════════════════════
// Root
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct TypedProgram {
    pub items: Vec<TypedItem>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Items
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct TypedItem {
    pub kind: TypedItemKind,
    pub span: Span,
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedItemKind {
    Function(TypedFnDecl),
    Struct(TypedStructDecl),
    Enum(TypedEnumDecl),
    Impl(TypedImplBlock),
    Trait(TypedTraitDecl),
    TypeAlias(TypedTypeAlias),
    Module(TypedModuleDecl),
    Use(crate::ast::UseDecl),
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedFnDecl {
    pub name: String,
    pub params: Vec<TypedFnParam>,
    pub return_type: TypeId,
    pub body: Option<TypedBlock>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedFnParam {
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedStructDecl {
    pub name: String,
    pub fields: Vec<TypedField>,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedField {
    pub name: String,
    pub ty: TypeId,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedEnumDecl {
    pub name: String,
    pub variants: Vec<TypedEnumVariant>,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedEnumVariant {
    pub name: String,
    pub fields: TypedEnumVariantKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedEnumVariantKind {
    Unit,
    Tuple(Vec<TypeId>),
    Struct(Vec<TypedField>),
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedImplBlock {
    pub type_name: TypeId,
    pub trait_name: Option<TypeId>,
    pub items: Vec<TypedItem>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedTraitDecl {
    pub name: String,
    pub items: Vec<TypedItem>,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedTypeAlias {
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedModuleDecl {
    pub name: String,
    pub items: Option<Vec<TypedItem>>,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct TypedStmt {
    pub kind: TypedStmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedStmtKind {
    Let {
        name: String,
        mutable: bool,
        ty: TypeId,
        initializer: Option<TypedExpr>,
    },
    Expr(TypedExpr),
    Return(Option<TypedExpr>),
    While {
        condition: TypedExpr,
        body: TypedBlock,
    },
    For {
        pattern: String,
        iterator: TypedExpr,
        body: TypedBlock,
    },
    Item(TypedItem),
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedBlock {
    pub stmts: Vec<TypedStmt>,
    pub tail_expr: Option<Box<TypedExpr>>,
    pub ty: TypeId,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// Expressions — every expression carries a resolved TypeId
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedExprKind {
    Literal(Literal),
    Identifier(String),
    Path(Vec<String>),
    BinaryOp {
        left: Box<TypedExpr>,
        op: BinOp,
        right: Box<TypedExpr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<TypedExpr>,
    },
    FnCall {
        function: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },
    MethodCall {
        receiver: Box<TypedExpr>,
        method: String,
        args: Vec<TypedExpr>,
    },
    FieldAccess {
        object: Box<TypedExpr>,
        field: String,
    },
    Index {
        object: Box<TypedExpr>,
        index: Box<TypedExpr>,
    },
    IfElse {
        condition: Box<TypedExpr>,
        then_block: TypedBlock,
        else_block: Option<TypedElseClause>,
    },
    Match {
        expr: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
    },
    Block(TypedBlock),
    Reference {
        mutable: bool,
        expr: Box<TypedExpr>,
    },
    TypeCast {
        expr: Box<TypedExpr>,
        target_type: TypeId,
    },
    ErrorPropagation(Box<TypedExpr>),
    Range {
        start: Option<Box<TypedExpr>>,
        end: Option<Box<TypedExpr>>,
    },
    Assignment {
        target: Box<TypedExpr>,
        op: AssignOp,
        value: Box<TypedExpr>,
    },
    Tuple(Vec<TypedExpr>),
    StructLiteral {
        name: Vec<String>,
        fields: Vec<TypedStructLiteralField>,
    },
    Closure {
        params: Vec<TypedFnParam>,
        body: Box<TypedExpr>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedElseClause {
    ElseBlock(TypedBlock),
    ElseIf(Box<TypedExpr>),
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    pub guard: Option<TypedExpr>,
    pub body: TypedExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedPattern {
    pub kind: TypedPatternKind,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypedPatternKind {
    Identifier(String),
    Literal(Literal),
    Wildcard,
    Tuple(Vec<TypedPattern>),
    Struct { name: Vec<String>, fields: Vec<TypedFieldPattern> },
    EnumVariant { path: Vec<String>, fields: Vec<TypedPattern> },
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedFieldPattern {
    pub name: String,
    pub pattern: Option<TypedPattern>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedStructLiteralField {
    pub name: String,
    pub value: TypedExpr,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════
// TastBuilder — converts AST + TypeChecker output into TAST
// ═══════════════════════════════════════════════════════════════

use crate::typeck::TypeChecker;
use crate::types::Type;
use crate::ast;

pub struct TastBuilder<'a> {
    #[allow(dead_code)]
    checker: &'a TypeChecker,
}

impl<'a> TastBuilder<'a> {
    pub fn new(checker: &'a TypeChecker) -> Self {
        TastBuilder { checker }
    }

    // ---------------------------------------------------------------
    // Type resolution helpers
    // ---------------------------------------------------------------

    /// Resolve an AST TypeExpr to a TypeId.
    fn resolve_type_expr(&self, ty: &ast::TypeExpr) -> TypeId {
        match &ty.kind {
            ast::TypeExprKind::Named(name) => self.resolve_type_name(name),
            ast::TypeExprKind::Inferred => TypeId::ERROR,
            _ => TypeId::ERROR, // complex types (tuples, arrays, refs, etc.) – future work
        }
    }

    /// Map a type name to a TypeId via primitives or symbol table.
    fn resolve_type_name(&self, name: &str) -> TypeId {
        if let Some(prim) = crate::symbol::prim_from_name(name) {
            if let Some(id) = self.checker.interner.lookup_prim(prim) {
                return id;
            }
        }
        if let Some(sym_id) = self.checker.symbols.lookup(name) {
            return self.checker.symbols.get_symbol(sym_id).ty;
        }
        TypeId::ERROR
    }

    /// Infer the type of a built expression from its kind.
    fn infer_expr_type(&self, kind: &TypedExprKind) -> TypeId {
        match kind {
            TypedExprKind::Literal(lit) => match lit {
                Literal::Int(_) => TypeId::INT64,
                Literal::Float(_) => TypeId::FLOAT64,
                Literal::Bool(_) => TypeId::BOOL,
                Literal::Char(_) => TypeId::CHAR,
                Literal::String(_) => TypeId::STRING,
            },
            TypedExprKind::Identifier(name) => {
                if let Some(sym_id) = self.checker.symbols.lookup(name) {
                    self.checker.symbols.get_symbol(sym_id).ty
                } else {
                    TypeId::ERROR
                }
            }
            TypedExprKind::BinaryOp { left, op, .. } => {
                // Comparison operators return Bool, arithmetic returns left operand type
                match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq
                    | BinOp::And | BinOp::Or => TypeId::BOOL,
                    _ => left.ty,
                }
            }
            TypedExprKind::UnaryOp { operand, .. } => operand.ty,
            TypedExprKind::FnCall { function, .. } => {
                // Special case: print/println always return Unit
                if let TypedExprKind::Identifier(name) = &function.kind {
                    if name == "print" || name == "println" {
                        return TypeId::UNIT;
                    }
                }
                let fn_ty = function.ty;
                let resolved = self.checker.interner.resolve(fn_ty);
                if let Type::Function { ret, .. } = resolved {
                    *ret
                } else {
                    TypeId::ERROR
                }
            }
            TypedExprKind::Block(block) => block.ty,
            TypedExprKind::IfElse { then_block, .. } => then_block.ty,
            TypedExprKind::Assignment { .. } => TypeId::UNIT,
            TypedExprKind::Reference { expr, .. } => expr.ty,
            TypedExprKind::Tuple(elems) => {
                if elems.is_empty() { TypeId::UNIT } else { TypeId::ERROR }
            }
            _ => TypeId::ERROR,
        }
    }

    // ---------------------------------------------------------------
    // Build methods
    // ---------------------------------------------------------------

    /// Walk the AST, creating typed counterparts.
    pub fn build(&self, program: &ast::Program) -> TypedProgram {
        TypedProgram {
            items: program.items.iter().map(|i| self.build_item(i)).collect(),
            span: program.span.clone(),
        }
    }

    fn build_item(&self, item: &ast::Item) -> TypedItem {
        let kind = match &item.kind {
            ast::ItemKind::Function(f) => TypedItemKind::Function(self.build_fn(f)),
            ast::ItemKind::Struct(s) => {
                let struct_ty = self.checker.symbols.lookup(&s.name)
                    .map(|sid| self.checker.symbols.get_symbol(sid).ty)
                    .unwrap_or(TypeId::ERROR);
                TypedItemKind::Struct(TypedStructDecl {
                    name: s.name.clone(),
                    fields: s.fields.iter().map(|f| TypedField {
                        name: f.name.clone(),
                        ty: self.resolve_type_expr(&f.ty),
                        visibility: f.visibility.clone(),
                        span: f.span.clone(),
                    }).collect(),
                    ty: struct_ty,
                    span: s.span.clone(),
                })
            }
            ast::ItemKind::Enum(e) => {
                let enum_ty = self.checker.symbols.lookup(&e.name)
                    .map(|sid| self.checker.symbols.get_symbol(sid).ty)
                    .unwrap_or(TypeId::ERROR);
                TypedItemKind::Enum(TypedEnumDecl {
                    name: e.name.clone(),
                    variants: e.variants.iter().map(|v| TypedEnumVariant {
                        name: v.name.clone(),
                        fields: match &v.fields {
                            ast::EnumVariantKind::Unit => TypedEnumVariantKind::Unit,
                            ast::EnumVariantKind::Tuple(types) => {
                                TypedEnumVariantKind::Tuple(
                                    types.iter().map(|t| self.resolve_type_expr(t)).collect(),
                                )
                            }
                            ast::EnumVariantKind::Struct(fields) => {
                                TypedEnumVariantKind::Struct(
                                    fields.iter().map(|f| TypedField {
                                        name: f.name.clone(),
                                        ty: self.resolve_type_expr(&f.ty),
                                        visibility: f.visibility.clone(),
                                        span: f.span.clone(),
                                    }).collect(),
                                )
                            }
                        },
                        span: v.span.clone(),
                    }).collect(),
                    ty: enum_ty,
                    span: e.span.clone(),
                })
            }
            ast::ItemKind::Impl(imp) => TypedItemKind::Impl(TypedImplBlock {
                type_name: self.resolve_type_expr(&imp.type_name),
                trait_name: imp.trait_name.as_ref().map(|t| self.resolve_type_expr(t)),
                items: imp.items.iter().map(|i| self.build_item(i)).collect(),
                span: imp.span.clone(),
            }),
            ast::ItemKind::Trait(t) => {
                let trait_ty = self.checker.symbols.lookup(&t.name)
                    .map(|sid| self.checker.symbols.get_symbol(sid).ty)
                    .unwrap_or(TypeId::ERROR);
                TypedItemKind::Trait(TypedTraitDecl {
                    name: t.name.clone(),
                    items: t.items.iter().map(|i| self.build_item(i)).collect(),
                    ty: trait_ty,
                    span: t.span.clone(),
                })
            }
            ast::ItemKind::TypeAlias(ta) => TypedItemKind::TypeAlias(TypedTypeAlias {
                name: ta.name.clone(),
                ty: self.resolve_type_expr(&ta.ty),
                span: ta.span.clone(),
            }),
            ast::ItemKind::Module(m) => TypedItemKind::Module(TypedModuleDecl {
                name: m.name.clone(),
                items: m.items.as_ref().map(|items| {
                    items.iter().map(|i| self.build_item(i)).collect()
                }),
                span: m.span.clone(),
            }),
            ast::ItemKind::Use(u) => TypedItemKind::Use(u.clone()),
        };

        TypedItem {
            kind,
            span: item.span.clone(),
            visibility: item.visibility.clone(),
            attributes: item.attributes.clone(),
        }
    }

    fn build_fn(&self, decl: &ast::FnDecl) -> TypedFnDecl {
        // Try to get function type from symbol table
        let (param_types, ret_type) = if let Some(sym_id) = self.checker.symbols.lookup(&decl.name) {
            let fn_sym = self.checker.symbols.get_symbol(sym_id);
            let resolved = self.checker.interner.resolve(fn_sym.ty);
            if let Type::Function { params, ret } = resolved {
                (params.clone(), *ret)
            } else {
                self.resolve_fn_from_ast(decl)
            }
        } else {
            self.resolve_fn_from_ast(decl)
        };

        TypedFnDecl {
            name: decl.name.clone(),
            params: decl.params.iter().enumerate().map(|(i, p)| {
                let name = match &p.kind {
                    ast::FnParamKind::Typed { name, .. } => name.clone(),
                    ast::FnParamKind::SelfOwned
                    | ast::FnParamKind::SelfRef
                    | ast::FnParamKind::SelfMutRef => "self".to_string(),
                };
                TypedFnParam {
                    name,
                    ty: param_types.get(i).copied().unwrap_or(TypeId::ERROR),
                    span: p.span.clone(),
                }
            }).collect(),
            return_type: ret_type,
            body: decl.body.as_ref().map(|b| self.build_block(b)),
            span: decl.span.clone(),
        }
    }

    /// Fallback: resolve function param/return types from AST annotations.
    fn resolve_fn_from_ast(&self, decl: &ast::FnDecl) -> (Vec<TypeId>, TypeId) {
        let params: Vec<TypeId> = decl.params.iter().map(|p| match &p.kind {
            ast::FnParamKind::Typed { ty, .. } => self.resolve_type_expr(ty),
            _ => TypeId::ERROR,
        }).collect();
        let ret = decl.return_type.as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(TypeId::UNIT);
        (params, ret)
    }

    fn build_block(&self, block: &ast::Block) -> TypedBlock {
        let stmts: Vec<TypedStmt> = block.stmts.iter().map(|s| self.build_stmt(s)).collect();
        let tail_expr = block.tail_expr.as_ref().map(|e| Box::new(self.build_expr(e)));
        let ty = if let Some(ref tail) = tail_expr {
            tail.ty
        } else if let Some(last) = stmts.last() {
            match &last.kind {
                TypedStmtKind::Return(Some(e)) => e.ty,
                TypedStmtKind::Expr(e) => e.ty,
                _ => TypeId::UNIT,
            }
        } else {
            TypeId::UNIT
        };
        TypedBlock {
            stmts,
            tail_expr,
            ty,
            span: block.span.clone(),
        }
    }

    fn build_stmt(&self, stmt: &ast::Stmt) -> TypedStmt {
        let kind = match &stmt.kind {
            ast::StmtKind::Let { name, mutable, ty, initializer } => {
                let name_str = match &name.kind {
                    ast::PatternKind::Identifier(s) => s.clone(),
                    _ => "_".to_string(),
                };
                let init_expr = initializer.as_ref().map(|e| self.build_expr(e));
                let resolved_ty = if let Some(ref t) = ty {
                    self.resolve_type_expr(t)
                } else if let Some(ref e) = init_expr {
                    e.ty
                } else {
                    TypeId::ERROR
                };
                TypedStmtKind::Let {
                    name: name_str,
                    mutable: *mutable,
                    ty: resolved_ty,
                    initializer: init_expr,
                }
            }
            ast::StmtKind::Expr(e) => TypedStmtKind::Expr(self.build_expr(e)),
            ast::StmtKind::Return(e) => TypedStmtKind::Return(e.as_ref().map(|e| self.build_expr(e))),
            ast::StmtKind::While { condition, body } => TypedStmtKind::While {
                condition: self.build_expr(condition),
                body: self.build_block(body),
            },
            ast::StmtKind::For { pattern, iterator, body } => {
                let pat_str = match &pattern.kind {
                    ast::PatternKind::Identifier(s) => s.clone(),
                    _ => "_".to_string(),
                };
                TypedStmtKind::For {
                    pattern: pat_str,
                    iterator: self.build_expr(iterator),
                    body: self.build_block(body),
                }
            }
            ast::StmtKind::Item(item) => TypedStmtKind::Item(self.build_item(item)),
        };

        TypedStmt {
            kind,
            span: stmt.span.clone(),
        }
    }

    fn build_expr(&self, expr: &ast::Expr) -> TypedExpr {
        let kind = match &expr.kind {
            ast::ExprKind::Literal(lit) => TypedExprKind::Literal(lit.clone()),
            ast::ExprKind::Identifier(s) => TypedExprKind::Identifier(s.clone()),
            ast::ExprKind::Path(p) => TypedExprKind::Path(p.clone()),
            ast::ExprKind::BinaryOp { left, op, right } => TypedExprKind::BinaryOp {
                left: Box::new(self.build_expr(left)),
                op: op.clone(),
                right: Box::new(self.build_expr(right)),
            },
            ast::ExprKind::UnaryOp { op, operand } => TypedExprKind::UnaryOp {
                op: op.clone(),
                operand: Box::new(self.build_expr(operand)),
            },
            ast::ExprKind::FnCall { function, args } => TypedExprKind::FnCall {
                function: Box::new(self.build_expr(function)),
                args: args.iter().map(|a| self.build_expr(a)).collect(),
            },
            ast::ExprKind::MethodCall { receiver, method, args } => TypedExprKind::MethodCall {
                receiver: Box::new(self.build_expr(receiver)),
                method: method.clone(),
                args: args.iter().map(|a| self.build_expr(a)).collect(),
            },
            ast::ExprKind::FieldAccess { object, field } => TypedExprKind::FieldAccess {
                object: Box::new(self.build_expr(object)),
                field: field.clone(),
            },
            ast::ExprKind::Index { object, index } => TypedExprKind::Index {
                object: Box::new(self.build_expr(object)),
                index: Box::new(self.build_expr(index)),
            },
            ast::ExprKind::Slice { object, start, end } => TypedExprKind::Index {
                object: Box::new(self.build_expr(object)),
                index: Box::new(TypedExpr {
                    kind: TypedExprKind::Range {
                        start: start.as_ref().map(|e| Box::new(self.build_expr(e))),
                        end: end.as_ref().map(|e| Box::new(self.build_expr(e))),
                    },
                    ty: TypeId::ERROR,
                    span: expr.span.clone(),
                }),
            },
            ast::ExprKind::IfElse { condition, then_block, else_block } => TypedExprKind::IfElse {
                condition: Box::new(self.build_expr(condition)),
                then_block: self.build_block(then_block),
                else_block: else_block.as_ref().map(|ec| match ec {
                    ast::ElseClause::ElseBlock(b) => TypedElseClause::ElseBlock(self.build_block(b)),
                    ast::ElseClause::ElseIf(e) => TypedElseClause::ElseIf(Box::new(self.build_expr(e))),
                }),
            },
            ast::ExprKind::Match { expr: e, arms } => TypedExprKind::Match {
                expr: Box::new(self.build_expr(e)),
                arms: arms.iter().map(|a| TypedMatchArm {
                    pattern: self.build_pattern(&a.pattern),
                    guard: a.guard.as_ref().map(|g| self.build_expr(g)),
                    body: self.build_expr(&a.body),
                    span: a.span.clone(),
                }).collect(),
            },
            ast::ExprKind::Block(b) => TypedExprKind::Block(self.build_block(b)),
            ast::ExprKind::Reference { mutable, expr: e } => TypedExprKind::Reference {
                mutable: *mutable,
                expr: Box::new(self.build_expr(e)),
            },
            ast::ExprKind::TypeCast { expr: e, target_type } => TypedExprKind::TypeCast {
                expr: Box::new(self.build_expr(e)),
                target_type: self.resolve_type_expr(target_type),
            },
            ast::ExprKind::ErrorPropagation(e) => TypedExprKind::ErrorPropagation(
                Box::new(self.build_expr(e)),
            ),
            ast::ExprKind::Range { start, end } => TypedExprKind::Range {
                start: start.as_ref().map(|e| Box::new(self.build_expr(e))),
                end: end.as_ref().map(|e| Box::new(self.build_expr(e))),
            },
            ast::ExprKind::Assignment { target, op, value } => TypedExprKind::Assignment {
                target: Box::new(self.build_expr(target)),
                op: op.clone(),
                value: Box::new(self.build_expr(value)),
            },
            ast::ExprKind::Tuple(elems) => TypedExprKind::Tuple(
                elems.iter().map(|e| self.build_expr(e)).collect(),
            ),
            ast::ExprKind::StructLiteral { name, fields } => TypedExprKind::StructLiteral {
                name: name.clone(),
                fields: fields.iter().map(|f| TypedStructLiteralField {
                    name: f.name.clone(),
                    value: self.build_expr(&f.value),
                    span: f.span.clone(),
                }).collect(),
            },
            ast::ExprKind::Closure { params, return_type: _, body } => TypedExprKind::Closure {
                params: params.iter().map(|p| TypedFnParam {
                    name: p.name.clone(),
                    ty: p.ty.as_ref().map(|t| self.resolve_type_expr(t)).unwrap_or(TypeId::ERROR),
                    span: p.span.clone(),
                }).collect(),
                body: Box::new(self.build_expr(body)),
            },
        };

        let ty = self.infer_expr_type(&kind);
        TypedExpr {
            kind,
            ty,
            span: expr.span.clone(),
        }
    }

    fn build_pattern(&self, pat: &ast::Pattern) -> TypedPattern {
        let kind = match &pat.kind {
            ast::PatternKind::Identifier(s) => TypedPatternKind::Identifier(s.clone()),
            ast::PatternKind::Literal(lit) => TypedPatternKind::Literal(lit.clone()),
            ast::PatternKind::Wildcard => TypedPatternKind::Wildcard,
            ast::PatternKind::Tuple(pats) => TypedPatternKind::Tuple(
                pats.iter().map(|p| self.build_pattern(p)).collect(),
            ),
            ast::PatternKind::Struct { name, fields } => TypedPatternKind::Struct {
                name: name.clone(),
                fields: fields.iter().map(|f| TypedFieldPattern {
                    name: f.name.clone(),
                    pattern: f.pattern.as_ref().map(|p| self.build_pattern(p)),
                    span: f.span.clone(),
                }).collect(),
            },
            ast::PatternKind::EnumVariant { path, fields } => TypedPatternKind::EnumVariant {
                path: path.clone(),
                fields: fields.iter().map(|p| self.build_pattern(p)).collect(),
            },
        };

        TypedPattern {
            kind,
            ty: TypeId::ERROR,
            span: pat.span.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeck;

    #[test]
    fn build_empty_program() {
        let (checker, _errors) = typeck::check("", "test.axon");
        let builder = TastBuilder::new(&checker);
        let program = crate::parse_source("", "test.axon").0;
        let typed = builder.build(&program);
        assert!(typed.items.is_empty());
    }

    #[test]
    fn build_simple_function() {
        let src = "fn main() -> Int64 { return 42; }";
        let (checker, _errors) = typeck::check(src, "test.axon");
        let builder = TastBuilder::new(&checker);
        let program = crate::parse_source(src, "test.axon").0;
        let typed = builder.build(&program);
        assert_eq!(typed.items.len(), 1);
        match &typed.items[0].kind {
            TypedItemKind::Function(f) => assert_eq!(f.name, "main"),
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn tast_serializes_to_json() {
        let src = "fn add(x: Int64, y: Int64) -> Int64 { return x; }";
        let (checker, _errors) = typeck::check(src, "test.axon");
        let builder = TastBuilder::new(&checker);
        let program = crate::parse_source(src, "test.axon").0;
        let typed = builder.build(&program);
        let json = serde_json::to_string_pretty(&typed).unwrap();
        assert!(json.contains("\"add\""));
    }
}
