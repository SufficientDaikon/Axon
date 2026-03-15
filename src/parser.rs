// parser.rs — Recursive descent parser for Axon
//
// Produces an AST from a token stream. Handles operator precedence,
// clear error messages with source locations, and multiple error recovery.

use crate::ast::*;
use crate::error::CompileError;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// The Axon parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub errors: Vec<CompileError>,
    #[allow(dead_code)]
    file: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: &str) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
            file: file.to_string(),
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Helpers
    // ═══════════════════════════════════════════════════════════

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens.last().unwrap() // Eof token
        })
    }

    fn peek_ahead(&self, offset: usize) -> &TokenKind {
        self.tokens
            .get(self.pos + offset)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.peek_token().span.clone()
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or_else(|| {
            self.tokens.last().unwrap().clone()
        });
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, CompileError> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            let span = self.current_span();
            Err(CompileError::new(
                "E0100",
                format!("expected '{}', found '{}'", expected.name(), self.peek()),
                span,
            ))
        }
    }

    fn expect_exact(&mut self, expected: TokenKind) -> Result<Token, CompileError> {
        if *self.peek() == expected {
            Ok(self.advance())
        } else {
            let span = self.current_span();
            Err(CompileError::new(
                "E0100",
                format!("expected '{}', found '{}'", expected.name(), self.peek()),
                span,
            ))
        }
    }

    #[allow(dead_code)]
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }

    fn at(&self, kind: &TokenKind) -> bool {
        *self.peek() == *kind
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn report(&mut self, error: CompileError) {
        self.errors.push(error);
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    /// Consume an identifier and return its name.
    fn expect_identifier(&mut self) -> Result<String, CompileError> {
        match self.peek().clone() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => {
                let span = self.current_span();
                Err(CompileError::new(
                    "E0101",
                    format!("expected identifier, found '{}'", self.peek()),
                    span,
                ))
            }
        }
    }

    /// Consume an identifier or a type name keyword and return it as a String.
    fn expect_name(&mut self) -> Result<String, CompileError> {
        match self.peek().clone() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            ref kind if is_type_keyword(kind) => {
                let name = kind.name().to_string();
                self.advance();
                Ok(name)
            }
            _ => {
                let span = self.current_span();
                Err(CompileError::new(
                    "E0101",
                    format!("expected identifier or type name, found '{}'", self.peek()),
                    span,
                ))
            }
        }
    }

    /// Skip tokens until we find something that looks like a recovery point.
    fn synchronize(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                TokenKind::Semicolon => {
                    self.advance();
                    return;
                }
                TokenKind::Fn | TokenKind::Val | TokenKind::Var
                | TokenKind::Model | TokenKind::Enum
                | TokenKind::Extend | TokenKind::Trait | TokenKind::Pub
                | TokenKind::Use | TokenKind::Mod | TokenKind::Type
                | TokenKind::Unsafe => return,
                TokenKind::RBrace => {
                    // Consume the closing brace so the outer loop doesn't
                    // get stuck trying to parse it as an item.
                    self.advance();
                    return;
                }
                _ => { self.advance(); }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Top-Level: Program
    // ═══════════════════════════════════════════════════════════

    /// Parse a complete Axon program.
    pub fn parse_program(&mut self) -> Program {
        let start = self.current_span();
        let mut items = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.report(e);
                    self.synchronize();
                }
            }
        }

        // Dedup: remove consecutive errors with the same message and line number.
        // Error recovery can sometimes report the same issue multiple times.
        self.errors.dedup_by(|b, a| {
            let same_msg = a.message == b.message;
            let same_line = match (&a.location, &b.location) {
                (Some(loc_a), Some(loc_b)) => loc_a.start.line == loc_b.start.line,
                (None, None) => true,
                _ => false,
            };
            same_msg && same_line
        });

        let end = self.current_span();
        Program {
            items,
            span: start.merge(&end),
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Items
    // ═══════════════════════════════════════════════════════════

    fn parse_item(&mut self) -> Result<Item, CompileError> {
        let start = self.current_span();
        let mut visibility = Visibility::Private;
        let mut attributes = Vec::new();

        // Parse attributes
        loop {
            match self.peek() {
                TokenKind::AtCpu => {
                    self.advance();
                    attributes.push(Attribute::Cpu);
                }
                TokenKind::AtGpu => {
                    self.advance();
                    attributes.push(Attribute::Gpu);
                }
                TokenKind::AtDevice => {
                    self.advance();
                    // Parse @device(expr)
                    self.expect_exact(TokenKind::LParen)?;
                    let expr = self.parse_expression()?;
                    self.expect_exact(TokenKind::RParen)?;
                    attributes.push(Attribute::Device(expr));
                }
                _ => break,
            }
        }

        // Parse visibility
        if self.at(&TokenKind::Pub) {
            self.advance();
            visibility = Visibility::Public;
        }

        let kind = match self.peek() {
            TokenKind::Fn => ItemKind::Function(self.parse_fn_decl()?),
            TokenKind::Model => ItemKind::Struct(self.parse_struct_decl()?),
            TokenKind::Enum => ItemKind::Enum(self.parse_enum_decl()?),
            TokenKind::Extend => ItemKind::Impl(self.parse_impl_block()?),
            TokenKind::Trait => ItemKind::Trait(self.parse_trait_decl()?),
            TokenKind::Type => ItemKind::TypeAlias(self.parse_type_alias()?),
            TokenKind::Mod => ItemKind::Module(self.parse_module_decl()?),
            TokenKind::Use => ItemKind::Use(self.parse_use_decl()?),
            TokenKind::Unsafe => {
                // `unsafe fn ...` or `unsafe { ... }`
                self.advance(); // consume `unsafe`
                match self.peek() {
                    TokenKind::Fn => ItemKind::Function(self.parse_fn_decl()?),
                    _ => {
                        return Err(CompileError::new(
                            "E0102",
                            format!("expected 'fn' after 'unsafe', found '{}'", self.peek()),
                            self.current_span(),
                        ));
                    }
                }
            }
            _ => {
                return Err(CompileError::new(
                    "E0102",
                    format!("expected item (fn, struct, model, enum, extend, impl, trait, type, mod, use), found '{}'", self.peek()),
                    self.current_span(),
                ).with_suggestion("top-level items must be declarations"));
            }
        };

        let end = self.current_span();
        Ok(Item {
            kind,
            span: start.merge(&end),
            visibility,
            attributes,
        })
    }

    // ── Function declaration ─────────────────────────────────

    fn parse_fn_decl(&mut self) -> Result<FnDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Fn)?;
        let name = self.expect_name()?;

        // Optional generics
        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Parameters
        self.expect_exact(TokenKind::LParen)?;
        let params = self.parse_fn_params()?;
        self.expect_exact(TokenKind::RParen)?;

        // Optional return type (: Type)
        let return_type = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body: block, expression shorthand (= expr;), or semicolon (trait method)
        let mut body = if self.at(&TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else if self.eat(&TokenKind::Eq) {
            // Expression shorthand: fn foo(): i32 = expr;
            let expr = self.parse_expression()?;
            self.expect_exact(TokenKind::Semicolon)?;
            let span = expr.span.clone();
            Some(Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Return(Some(expr)),
                    span: span.clone(),
                }],
                tail_expr: None,
                span,
            })
        } else {
            self.expect_exact(TokenKind::Semicolon)?;
            None
        };

        // Implicit return: if the function has a return type and the body's
        // last statement is a bare expression (without ;), convert to return
        if return_type.is_some() {
            if let Some(ref mut block) = body {
                if let Some(last) = block.stmts.last() {
                    if let StmtKind::Expr(expr) = &last.kind {
                        if !matches!(&expr.kind,
                            ExprKind::Assignment { .. }
                            | ExprKind::IfElse { .. }
                            | ExprKind::Match { .. }
                            | ExprKind::Block(_)
                        ) {
                            let last_stmt = block.stmts.pop().unwrap();
                            if let StmtKind::Expr(expr) = last_stmt.kind {
                                let span = expr.span.clone();
                                block.stmts.push(Stmt {
                                    kind: StmtKind::Return(Some(expr)),
                                    span,
                                });
                            }
                        }
                    }
                }
            }
        }

        let end = self.current_span();
        Ok(FnDecl {
            name,
            generics,
            params,
            return_type,
            body,
            span: start.merge(&end),
        })
    }

    fn parse_fn_params(&mut self) -> Result<Vec<FnParam>, CompileError> {
        let mut params = Vec::new();
        if self.at(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let param_start = self.current_span();

            // Check for self parameters
            if self.at(&TokenKind::AmpMut) && matches!(self.peek_ahead(1), TokenKind::SelfValue) {
                self.advance(); // &mut
                self.advance(); // self
                params.push(FnParam {
                    kind: FnParamKind::SelfMutRef,
                    span: param_start.merge(&self.current_span()),
                });
            } else if self.at(&TokenKind::Amp) && matches!(self.peek_ahead(1), TokenKind::SelfValue) {
                self.advance(); // &
                self.advance(); // self
                params.push(FnParam {
                    kind: FnParamKind::SelfRef,
                    span: param_start.merge(&self.current_span()),
                });
            } else if self.at(&TokenKind::SelfValue) {
                self.advance(); // self
                params.push(FnParam {
                    kind: FnParamKind::SelfOwned,
                    span: param_start.merge(&self.current_span()),
                });
            } else {
                // Normal parameter: name: Type [= default]
                let name = self.expect_identifier()?;
                self.expect_exact(TokenKind::Colon)?;
                let ty = self.parse_type()?;

                let default = if self.eat(&TokenKind::Eq) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                params.push(FnParam {
                    kind: FnParamKind::Typed { name, ty, default },
                    span: param_start.merge(&self.current_span()),
                });
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, CompileError> {
        self.expect_exact(TokenKind::Lt)?;
        let mut params = Vec::new();

        while !self.at(&TokenKind::Gt) && !self.is_at_end() {
            let start = self.current_span();
            let name = self.expect_name()?;

            let mut bounds = Vec::new();
            if self.eat(&TokenKind::Colon) {
                bounds.push(self.parse_type()?);
                while self.eat(&TokenKind::Plus) {
                    bounds.push(self.parse_type()?);
                }
            }

            params.push(GenericParam {
                name,
                bounds,
                span: start.merge(&self.current_span()),
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::Gt)?;
        Ok(params)
    }

    // ── Struct declaration ───────────────────────────────────

    fn parse_struct_decl(&mut self) -> Result<StructDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Model)?;
        let name = self.expect_name()?;

        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect_exact(TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            let field_start = self.current_span();
            let vis = if self.eat(&TokenKind::Pub) {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let fname = self.expect_identifier()?;
            self.expect_exact(TokenKind::Colon)?;
            let ty = self.parse_type()?;

            fields.push(StructField {
                name: fname,
                ty,
                visibility: vis,
                span: field_start.merge(&self.current_span()),
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(StructDecl {
            name,
            generics,
            fields,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Enum declaration ─────────────────────────────────────

    fn parse_enum_decl(&mut self) -> Result<EnumDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Enum)?;
        let name = self.expect_name()?;

        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect_exact(TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            let var_start = self.current_span();
            let vname = self.expect_name()?;

            let fields = if self.at(&TokenKind::LParen) {
                self.advance();
                let mut types = Vec::new();
                while !self.at(&TokenKind::RParen) && !self.is_at_end() {
                    types.push(self.parse_type()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RParen)?;
                EnumVariantKind::Tuple(types)
            } else if self.at(&TokenKind::LBrace) {
                self.advance();
                let mut sfields = Vec::new();
                while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
                    let fs = self.current_span();
                    let fname = self.expect_identifier()?;
                    self.expect_exact(TokenKind::Colon)?;
                    let ty = self.parse_type()?;
                    sfields.push(StructField {
                        name: fname,
                        ty,
                        visibility: Visibility::Private,
                        span: fs.merge(&self.current_span()),
                    });
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RBrace)?;
                EnumVariantKind::Struct(sfields)
            } else {
                EnumVariantKind::Unit
            };

            variants.push(EnumVariant {
                name: vname,
                fields,
                span: var_start.merge(&self.current_span()),
            });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(EnumDecl {
            name,
            generics,
            variants,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Impl block ───────────────────────────────────────────

    fn parse_impl_block(&mut self) -> Result<ImplBlock, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Extend)?;

        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let first_type = self.parse_type()?;

        // Check for `impl Trait for Type`
        let (trait_name, type_name) = if self.at(&TokenKind::For) {
            self.advance();
            let tn = self.parse_type()?;
            (Some(first_type), tn)
        } else {
            (None, first_type)
        };

        self.expect_exact(TokenKind::LBrace)?;
        let mut items = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.report(e);
                    self.synchronize();
                }
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(ImplBlock {
            type_name,
            trait_name,
            generics,
            items,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Trait declaration ────────────────────────────────────

    fn parse_trait_decl(&mut self) -> Result<TraitDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Trait)?;
        let name = self.expect_name()?;

        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let mut supertraits = Vec::new();
        if self.eat(&TokenKind::Colon) {
            supertraits.push(self.parse_type()?);
            while self.eat(&TokenKind::Plus) {
                supertraits.push(self.parse_type()?);
            }
        }

        self.expect_exact(TokenKind::LBrace)?;
        let mut items = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.report(e);
                    self.synchronize();
                }
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(TraitDecl {
            name,
            generics,
            supertraits,
            items,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Type alias ───────────────────────────────────────────

    fn parse_type_alias(&mut self) -> Result<TypeAliasDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Type)?;
        let name = self.expect_name()?;

        let generics = if self.at(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect_exact(TokenKind::Eq)?;
        let ty = self.parse_type()?;
        self.expect_exact(TokenKind::Semicolon)?;

        Ok(TypeAliasDecl {
            name,
            generics,
            ty,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Module declaration ───────────────────────────────────

    fn parse_module_decl(&mut self) -> Result<ModuleDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Mod)?;
        let name = self.expect_identifier()?;

        let items = if self.at(&TokenKind::LBrace) {
            self.advance();
            let mut inner = Vec::new();
            while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
                match self.parse_item() {
                    Ok(item) => inner.push(item),
                    Err(e) => {
                        self.report(e);
                        self.synchronize();
                    }
                }
            }
            self.expect_exact(TokenKind::RBrace)?;
            Some(inner)
        } else {
            self.expect_exact(TokenKind::Semicolon)?;
            None
        };

        Ok(ModuleDecl {
            name,
            items,
            span: start.merge(&self.current_span()),
        })
    }

    // ── Use declaration ──────────────────────────────────────

    fn parse_use_decl(&mut self) -> Result<UseDecl, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Use)?;

        let mut path = Vec::new();
        path.push(self.expect_name()?);

        while self.eat(&TokenKind::Dot) {
            path.push(self.expect_name()?);
        }

        let alias = if self.at(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect_exact(TokenKind::Semicolon)?;
        Ok(UseDecl {
            path,
            alias,
            span: start.merge(&self.current_span()),
        })
    }

    // ═══════════════════════════════════════════════════════════
    // Statements
    // ═══════════════════════════════════════════════════════════

    fn parse_statement(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();

        match self.peek() {
            TokenKind::Val | TokenKind::Var => self.parse_let_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            // Items that can appear inside blocks
            TokenKind::Fn | TokenKind::Model | TokenKind::Enum
            | TokenKind::Extend | TokenKind::Trait | TokenKind::Type
            | TokenKind::Mod | TokenKind::Use | TokenKind::Pub => {
                let item = self.parse_item()?;
                Ok(Stmt {
                    kind: StmtKind::Item(item),
                    span: start.merge(&self.current_span()),
                })
            }
            _ => {
                let expr = self.parse_expression()?;

                // Check if this is an assignment
                let kind = match self.peek() {
                    TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq
                    | TokenKind::StarEq | TokenKind::SlashEq => {
                        let op = match self.advance().kind {
                            TokenKind::Eq => AssignOp::Assign,
                            TokenKind::PlusEq => AssignOp::AddAssign,
                            TokenKind::MinusEq => AssignOp::SubAssign,
                            TokenKind::StarEq => AssignOp::MulAssign,
                            TokenKind::SlashEq => AssignOp::DivAssign,
                            _ => unreachable!(),
                        };
                        let value = self.parse_expression()?;
                        let aspan = start.merge(&self.current_span());
                        StmtKind::Expr(Expr {
                            kind: ExprKind::Assignment {
                                target: Box::new(expr),
                                op,
                                value: Box::new(value),
                            },
                            span: aspan,
                        })
                    }
                    _ => StmtKind::Expr(expr),
                };

                // Expect semicolon for expression statements
                // But not for if/match/block expressions that end with }
                let needs_semi = match &kind {
                    StmtKind::Expr(e) => !matches!(
                        &e.kind,
                        ExprKind::IfElse { .. }
                            | ExprKind::Match { .. }
                            | ExprKind::Block(_)
                            | ExprKind::Assignment { .. }
                    ),
                    _ => true,
                };

                // For assignments we do need a semicolon
                let needs_semi = match &kind {
                    StmtKind::Expr(e) => match &e.kind {
                        ExprKind::Assignment { .. } => true,
                        ExprKind::IfElse { .. } | ExprKind::Match { .. } | ExprKind::Block(_) => false,
                        _ => true,
                    },
                    _ => needs_semi,
                };

                if needs_semi {
                    // Allow omitting semicolon before } (implicit return)
                    if !self.at(&TokenKind::RBrace) {
                        self.expect_exact(TokenKind::Semicolon)?;
                    }
                }

                Ok(Stmt {
                    kind,
                    span: start.merge(&self.current_span()),
                })
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();

        // Accept: val (immutable), var (mutable)
        let mutable = if self.eat(&TokenKind::Var) {
            true
        } else {
            self.expect_exact(TokenKind::Val)?;
            false
        };

        let pattern = self.parse_pattern()?;

        let ty = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.eat(&TokenKind::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_exact(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Let {
                name: pattern,
                mutable,
                ty,
                initializer,
            },
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Return)?;

        let value = if !self.at(&TokenKind::Semicolon) && !self.at(&TokenKind::RBrace) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_exact(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Return(value),
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::While { condition, body },
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::For)?;
        let pattern = self.parse_pattern()?;
        self.expect_exact(TokenKind::In)?;
        let iterator = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt {
            kind: StmtKind::For {
                pattern,
                iterator,
                body,
            },
            span: start.merge(&self.current_span()),
        })
    }

    // ═══════════════════════════════════════════════════════════
    // Blocks
    // ═══════════════════════════════════════════════════════════

    fn parse_block(&mut self) -> Result<Block, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::LBrace)?;
        let mut stmts = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.report(e);
                    self.synchronize();
                }
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(Block {
            stmts,
            tail_expr: None,
            span: start.merge(&self.current_span()),
        })
    }

    // ═══════════════════════════════════════════════════════════
    // Expressions — Pratt parser with precedence climbing
    // ═══════════════════════════════════════════════════════════

    fn parse_expression(&mut self) -> Result<Expr, CompileError> {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
            self.parse_pipe_expr()
        })
    }

    // Precedence (low → high):
    // 0. |> (pipe)
    // 1. || (or)
    // 2. && (and)
    // 3. ==, !=, <, >, <=, >= (comparison)
    // 4. + - (add/sub)
    // 5. * / % @ (mul/div/mod/matmul)
    // 6. as (type cast)
    // 7. unary: -, !, &, &mut
    // 8. postfix: ?, (), [], ., ::

    fn parse_pipe_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_or_expr()?;

        while self.at(&TokenKind::Pipe) {
            let start = left.span.clone();
            self.advance();
            let right = self.parse_or_expr()?;
            let span = start.merge(&right.span);

            // Desugar: a |> f(b, c) → f(a, b, c)
            //          a |> f        → f(a)
            //          a |> obj.method(b) → obj.method(a, b)
            left = match right.kind {
                ExprKind::FnCall { function, mut args } => {
                    args.insert(0, left);
                    Expr {
                        kind: ExprKind::FnCall { function, args },
                        span,
                    }
                }
                ExprKind::MethodCall { receiver, method, mut args } => {
                    args.insert(0, left);
                    Expr {
                        kind: ExprKind::MethodCall { receiver, method, args },
                        span,
                    }
                }
                // Bare identifier: a |> f → f(a)
                _ => {
                    Expr {
                        kind: ExprKind::FnCall {
                            function: Box::new(right),
                            args: vec![left],
                        },
                        span,
                    }
                }
            };
        }

        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_and_expr()?;

        while self.at(&TokenKind::OrOr) {
            let start = left.span.clone();
            self.advance();
            let right = self.parse_and_expr()?;
            let span = start.merge(&right.span);
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Or,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_comparison_expr()?;

        while self.at(&TokenKind::AndAnd) {
            let start = left.span.clone();
            self.advance();
            let right = self.parse_comparison_expr()?;
            let span = start.merge(&right.span);
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::And,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_range_expr()?;

        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::NotEq => BinOp::NotEq,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::GtEq => BinOp::GtEq,
                _ => break,
            };
            let start = left.span.clone();
            self.advance();
            let right = self.parse_range_expr()?;
            let span = start.merge(&right.span);
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_range_expr(&mut self) -> Result<Expr, CompileError> {
        let left = self.parse_additive_expr()?;

        if self.at(&TokenKind::DotDot) {
            let start = left.span.clone();
            self.advance();
            if self.at(&TokenKind::Semicolon)
                || self.at(&TokenKind::RParen)
                || self.at(&TokenKind::RBracket)
                || self.at(&TokenKind::Comma)
            {
                return Ok(Expr {
                    kind: ExprKind::Range {
                        start: Some(Box::new(left)),
                        end: None,
                    },
                    span: start.merge(&self.current_span()),
                });
            }
            let right = self.parse_additive_expr()?;
            let span = start.merge(&right.span);
            return Ok(Expr {
                kind: ExprKind::Range {
                    start: Some(Box::new(left)),
                    end: Some(Box::new(right)),
                },
                span,
            });
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_multiplicative_expr()?;

        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            let start = left.span.clone();
            self.advance();
            let right = self.parse_multiplicative_expr()?;
            let span = start.merge(&right.span);
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, CompileError> {
        let mut left = self.parse_cast_expr()?;

        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                TokenKind::At => BinOp::MatMul,
                _ => break,
            };
            let start = left.span.clone();
            self.advance();
            let right = self.parse_cast_expr()?;
            let span = start.merge(&right.span);
            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(left)
    }

    fn parse_cast_expr(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_unary_expr()?;

        while self.at(&TokenKind::As) {
            let start = expr.span.clone();
            self.advance();
            let target_type = self.parse_type()?;
            let span = start.merge(&target_type.span);
            expr = Expr {
                kind: ExprKind::TypeCast {
                    expr: Box::new(expr),
                    target_type,
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();

        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(&operand.span);
                Ok(Expr {
                    kind: ExprKind::UnaryOp {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    span,
                })
            }
            TokenKind::Bang => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(&operand.span);
                Ok(Expr {
                    kind: ExprKind::UnaryOp {
                        op: UnaryOp::Not,
                        operand: Box::new(operand),
                    },
                    span,
                })
            }
            TokenKind::AmpMut => {
                self.advance();
                let operand = self.parse_unary_expr()?;
                let span = start.merge(&operand.span);
                Ok(Expr {
                    kind: ExprKind::Reference {
                        mutable: true,
                        expr: Box::new(operand),
                    },
                    span,
                })
            }
            TokenKind::Amp => {
                self.advance();
                // Check for `&mut`
                if self.at(&TokenKind::Mut) {
                    self.advance();
                    let operand = self.parse_unary_expr()?;
                    let span = start.merge(&operand.span);
                    Ok(Expr {
                        kind: ExprKind::Reference {
                            mutable: true,
                            expr: Box::new(operand),
                        },
                        span,
                    })
                } else {
                    let operand = self.parse_unary_expr()?;
                    let span = start.merge(&operand.span);
                    Ok(Expr {
                        kind: ExprKind::Reference {
                            mutable: false,
                            expr: Box::new(operand),
                        },
                        span,
                    })
                }
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, CompileError> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.peek() {
                // Function call: expr(args)
                TokenKind::LParen => {
                    let start = expr.span.clone();
                    self.advance();
                    let args = self.parse_call_args()?;
                    self.expect_exact(TokenKind::RParen)?;
                    let span = start.merge(&self.current_span());
                    expr = Expr {
                        kind: ExprKind::FnCall {
                            function: Box::new(expr),
                            args,
                        },
                        span,
                    };
                }
                // Index or slice: expr[index]
                TokenKind::LBracket => {
                    let start = expr.span.clone();
                    self.advance();
                    let index = self.parse_expression()?;

                    // Check for slice syntax: expr[start..end]
                    if self.at(&TokenKind::DotDot) {
                        self.advance();
                        let end_expr = if !self.at(&TokenKind::RBracket) {
                            Some(Box::new(self.parse_expression()?))
                        } else {
                            None
                        };
                        self.expect_exact(TokenKind::RBracket)?;
                        let span = start.merge(&self.current_span());
                        expr = Expr {
                            kind: ExprKind::Slice {
                                object: Box::new(expr),
                                start: Some(Box::new(index)),
                                end: end_expr,
                            },
                            span,
                        };
                    } else {
                        self.expect_exact(TokenKind::RBracket)?;
                        let span = start.merge(&self.current_span());
                        expr = Expr {
                            kind: ExprKind::Index {
                                object: Box::new(expr),
                                index: Box::new(index),
                            },
                            span,
                        };
                    }
                }
                // Field access or method call: expr.field or expr.method(args)
                TokenKind::Dot => {
                    let start = expr.span.clone();
                    self.advance();
                    let field_name = self.expect_name()?;

                    if self.at(&TokenKind::LParen) {
                        self.advance();
                        let args = self.parse_call_args()?;
                        self.expect_exact(TokenKind::RParen)?;
                        let span = start.merge(&self.current_span());
                        expr = Expr {
                            kind: ExprKind::MethodCall {
                                receiver: Box::new(expr),
                                method: field_name,
                                args,
                            },
                            span,
                        };
                    } else {
                        let span = start.merge(&self.current_span());
                        expr = Expr {
                            kind: ExprKind::FieldAccess {
                                object: Box::new(expr),
                                field: field_name,
                            },
                            span,
                        };
                    }
                }
                // Error propagation: expr?
                TokenKind::Question => {
                    let start = expr.span.clone();
                    self.advance();
                    let span = start.merge(&self.current_span());
                    expr = Expr {
                        kind: ExprKind::ErrorPropagation(Box::new(expr)),
                        span,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, CompileError> {
        let mut args = Vec::new();
        if self.at(&TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();

        match self.peek().clone() {
            // ── Literals ─────────────────────────────────────
            TokenKind::IntLiteral(n) => {
                let n = n;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Int(n)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::FloatLiteral(n) => {
                let n = n;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Float(n)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::String(s)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::CharLiteral(c) => {
                let c = c;
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Char(c)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Bool(true)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Literal(Literal::Bool(false)),
                    span: start.merge(&self.current_span()),
                })
            }

            // ── SelfValue ────────────────────────────────────
            TokenKind::SelfValue => {
                self.advance();
                Ok(Expr {
                    kind: ExprKind::Identifier("self".to_string()),
                    span: start.merge(&self.current_span()),
                })
            }

            // ── If expression ────────────────────────────────
            TokenKind::If => self.parse_if_expr(),

            // ── Match expression ─────────────────────────────
            TokenKind::Match => self.parse_match_expr(),

            // ── Block expression ─────────────────────────────
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                let span = block.span.clone();
                Ok(Expr {
                    kind: ExprKind::Block(block),
                    span,
                })
            }

            // ── Parenthesized expression or tuple ────────────
            TokenKind::LParen => {
                self.advance();
                if self.at(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr {
                        kind: ExprKind::Tuple(Vec::new()),
                        span: start.merge(&self.current_span()),
                    });
                }

                let first = self.parse_expression()?;

                if self.at(&TokenKind::Comma) {
                    // It's a tuple
                    let mut exprs = vec![first];
                    while self.eat(&TokenKind::Comma) {
                        if self.at(&TokenKind::RParen) {
                            break;
                        }
                        exprs.push(self.parse_expression()?);
                    }
                    self.expect_exact(TokenKind::RParen)?;
                    Ok(Expr {
                        kind: ExprKind::Tuple(exprs),
                        span: start.merge(&self.current_span()),
                    })
                } else {
                    // Parenthesized expression
                    self.expect_exact(TokenKind::RParen)?;
                    Ok(first)
                }
            }

            // ── Array literal: [expr, expr, ...] ─────────────
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.at(&TokenKind::RBracket) && !self.is_at_end() {
                    elements.push(self.parse_expression()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RBracket)?;
                // Represent as a function call to a built-in array constructor
                Ok(Expr {
                    kind: ExprKind::FnCall {
                        function: Box::new(Expr {
                            kind: ExprKind::Identifier("__array".to_string()),
                            span: start.clone(),
                        }),
                        args: elements,
                    },
                    span: start.merge(&self.current_span()),
                })
            }

            // ── Identifier or path ───────────────────────────
            TokenKind::Identifier(_) => self.parse_identifier_or_path(),

            // ── Type keywords as identifiers in expressions ──
            ref kind if is_type_keyword(kind) => self.parse_identifier_or_path(),

            _ => {
                Err(CompileError::new(
                    "E0103",
                    format!("expected expression, found '{}'", self.peek()),
                    start,
                ).with_suggestion("expressions include literals, identifiers, function calls, and operators"))
            }
        }
    }

    fn parse_identifier_or_path(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        let first = self.expect_name()?;

        // Check for path: A.b.c (dot paths only when first segment is uppercase)
        let is_dot_path = self.at(&TokenKind::Dot)
            && first.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        if is_dot_path {
            let mut segments = vec![first];
            while self.at(&TokenKind::Dot)
                && segments[0].chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && self.eat(&TokenKind::Dot) {
                segments.push(self.expect_name()?);
            }

            // Check for struct literal: Path { field: value, ... }
            // Only if followed by { and then identifier:
            if self.at(&TokenKind::LBrace) && self.looks_like_struct_literal() {
                return self.parse_struct_literal(segments, start);
            }

            let span = start.merge(&self.current_span());
            return Ok(Expr {
                kind: ExprKind::Path(segments),
                span,
            });
        }

        // Check for struct literal: Name { field: value, ... }
        if self.at(&TokenKind::LBrace) && self.looks_like_struct_literal() {
            return self.parse_struct_literal(vec![first], start);
        }

        let span = start.merge(&self.current_span());
        Ok(Expr {
            kind: ExprKind::Identifier(first),
            span,
        })
    }

    /// Heuristic: does this look like a struct literal `Name { field: val }` rather than
    /// a block `name { stmt; stmt; }`?
    fn looks_like_struct_literal(&self) -> bool {
        // Look ahead: if we see `{ identifier :` or `{ }` it's likely a struct literal
        if self.peek_ahead(0) != &TokenKind::LBrace {
            return false;
        }

        match self.peek_ahead(1) {
            TokenKind::RBrace => true, // empty struct literal `Foo {}`
            TokenKind::Identifier(_) => {
                // Check if followed by `:`
                matches!(self.peek_ahead(2), TokenKind::Colon)
            }
            _ => false,
        }
    }

    fn parse_struct_literal(
        &mut self,
        name: Vec<String>,
        start: Span,
    ) -> Result<Expr, CompileError> {
        self.expect_exact(TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            let fstart = self.current_span();
            let fname = self.expect_identifier()?;
            self.expect_exact(TokenKind::Colon)?;
            let value = self.parse_expression()?;
            fields.push(StructLiteralField {
                name: fname,
                value,
                span: fstart.merge(&self.current_span()),
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(Expr {
            kind: ExprKind::StructLiteral { name, fields },
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_block = if self.eat(&TokenKind::Else) {
            if self.at(&TokenKind::If) {
                let else_if = self.parse_if_expr()?;
                Some(ElseClause::ElseIf(Box::new(else_if)))
            } else {
                let block = self.parse_block()?;
                Some(ElseClause::ElseBlock(block))
            }
        } else {
            None
        };

        Ok(Expr {
            kind: ExprKind::IfElse {
                condition: Box::new(condition),
                then_block,
                else_block,
            },
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_match_expr(&mut self) -> Result<Expr, CompileError> {
        let start = self.current_span();
        self.expect_exact(TokenKind::Match)?;
        let expr = self.parse_expression()?;
        self.expect_exact(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
            let arm_start = self.current_span();
            let pattern = self.parse_pattern()?;

            let guard = if self.at(&TokenKind::If) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect_exact(TokenKind::FatArrow)?;
            let body = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_start.merge(&self.current_span()),
            });

            // Match arms are separated by commas
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::RBrace)?;
        Ok(Expr {
            kind: ExprKind::Match {
                expr: Box::new(expr),
                arms,
            },
            span: start.merge(&self.current_span()),
        })
    }

    // ═══════════════════════════════════════════════════════════
    // Types
    // ═══════════════════════════════════════════════════════════

    fn parse_type(&mut self) -> Result<TypeExpr, CompileError> {
        let start = self.current_span();

        match self.peek().clone() {
            // Reference types: &T, &mut T
            TokenKind::Amp => {
                self.advance();
                let mutable = self.eat(&TokenKind::Mut);
                let inner = self.parse_type()?;
                let span = start.merge(&inner.span);
                Ok(TypeExpr {
                    kind: TypeExprKind::Reference {
                        mutable,
                        inner: Box::new(inner),
                    },
                    span,
                })
            }
            TokenKind::AmpMut => {
                self.advance();
                let inner = self.parse_type()?;
                let span = start.merge(&inner.span);
                Ok(TypeExpr {
                    kind: TypeExprKind::Reference {
                        mutable: true,
                        inner: Box::new(inner),
                    },
                    span,
                })
            }

            // Function type: fn(A, B) -> C
            TokenKind::Fn => {
                self.advance();
                self.expect_exact(TokenKind::LParen)?;
                let mut params = Vec::new();
                while !self.at(&TokenKind::RParen) && !self.is_at_end() {
                    params.push(self.parse_type()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RParen)?;
                let return_type = if self.eat(&TokenKind::Colon) {
                    self.parse_type()?
                } else {
                    TypeExpr {
                        kind: TypeExprKind::Tuple(Vec::new()),
                        span: self.current_span(),
                    }
                };
                let span = start.merge(&return_type.span);
                Ok(TypeExpr {
                    kind: TypeExprKind::Function {
                        params,
                        return_type: Box::new(return_type),
                    },
                    span,
                })
            }

            // Tuple type: (A, B, C)
            TokenKind::LParen => {
                self.advance();
                let mut types = Vec::new();
                while !self.at(&TokenKind::RParen) && !self.is_at_end() {
                    types.push(self.parse_type()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RParen)?;
                Ok(TypeExpr {
                    kind: TypeExprKind::Tuple(types),
                    span: start.merge(&self.current_span()),
                })
            }

            // Array type: [T; N]
            TokenKind::LBracket => {
                self.advance();
                let element = self.parse_type()?;
                self.expect_exact(TokenKind::Semicolon)?;
                let size_tok = self.expect(&TokenKind::IntLiteral(0))?;
                let size = match size_tok.kind {
                    TokenKind::IntLiteral(n) => n as usize,
                    _ => 0,
                };
                self.expect_exact(TokenKind::RBracket)?;
                Ok(TypeExpr {
                    kind: TypeExprKind::Array {
                        element: Box::new(element),
                        size,
                    },
                    span: start.merge(&self.current_span()),
                })
            }

            // Named types, generic types, tensor types, path types
            _ => self.parse_named_type(),
        }
    }

    fn parse_named_type(&mut self) -> Result<TypeExpr, CompileError> {
        let start = self.current_span();
        let name = self.expect_name()?;

        // Check for path: A.B.C
        let mut path = vec![name.clone()];
        while self.at(&TokenKind::Dot) && !self.is_at_end() {
            self.advance();
            path.push(self.expect_name()?);
        }

        // Check for generic args: <...>
        if self.at(&TokenKind::Lt) {
            // Special case: Tensor<DType, [shape]>
            if name == "Tensor" && path.len() == 1 {
                return self.parse_tensor_type(start);
            }

            self.advance(); // <
            let mut args = Vec::new();

            while !self.at(&TokenKind::Gt) && !self.is_at_end() {
                // Type argument can be a type, a shape array, or an integer literal
                if self.at(&TokenKind::LBracket) {
                    let shape = self.parse_shape_dims()?;
                    args.push(TypeArg::Shape(shape));
                } else if matches!(self.peek(), TokenKind::IntLiteral(_)) {
                    // Integer literal as type arg (e.g., Linear<784, 256>)
                    let shape_dim = match self.peek().clone() {
                        TokenKind::IntLiteral(n) => {
                            self.advance();
                            ShapeDim::Constant(n)
                        }
                        _ => unreachable!(),
                    };
                    args.push(TypeArg::Shape(vec![shape_dim]));
                } else {
                    let ty = self.parse_type()?;
                    args.push(TypeArg::Type(ty));
                }
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }

            self.expect_exact(TokenKind::Gt)?;

            let full_name = if path.len() > 1 {
                path.join(".")
            } else {
                name
            };

            return Ok(TypeExpr {
                kind: TypeExprKind::Generic {
                    name: full_name,
                    args,
                },
                span: start.merge(&self.current_span()),
            });
        }

        // Simple named type or path type
        if path.len() > 1 {
            Ok(TypeExpr {
                kind: TypeExprKind::Path(path),
                span: start.merge(&self.current_span()),
            })
        } else {
            Ok(TypeExpr {
                kind: TypeExprKind::Named(name),
                span: start.merge(&self.current_span()),
            })
        }
    }

    fn parse_tensor_type(&mut self, start: Span) -> Result<TypeExpr, CompileError> {
        self.expect_exact(TokenKind::Lt)?;
        let dtype = self.parse_type()?;
        self.expect_exact(TokenKind::Comma)?;
        let shape = self.parse_shape_dims()?;
        self.expect_exact(TokenKind::Gt)?;

        Ok(TypeExpr {
            kind: TypeExprKind::Tensor {
                dtype: Box::new(dtype),
                shape,
            },
            span: start.merge(&self.current_span()),
        })
    }

    fn parse_shape_dims(&mut self) -> Result<Vec<ShapeDim>, CompileError> {
        self.expect_exact(TokenKind::LBracket)?;
        let mut dims = Vec::new();

        while !self.at(&TokenKind::RBracket) && !self.is_at_end() {
            match self.peek().clone() {
                TokenKind::Question => {
                    self.advance();
                    dims.push(ShapeDim::Dynamic);
                }
                TokenKind::IntLiteral(n) => {
                    self.advance();
                    dims.push(ShapeDim::Constant(n));
                }
                TokenKind::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    dims.push(ShapeDim::Named(name));
                }
                _ => {
                    return Err(CompileError::new(
                        "E0110",
                        format!(
                            "expected dimension (integer, '?', or name), found '{}'",
                            self.peek()
                        ),
                        self.current_span(),
                    ));
                }
            }

            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_exact(TokenKind::RBracket)?;
        Ok(dims)
    }

    // ═══════════════════════════════════════════════════════════
    // Patterns
    // ═══════════════════════════════════════════════════════════

    fn parse_pattern(&mut self) -> Result<Pattern, CompileError> {
        let start = self.current_span();

        match self.peek().clone() {
            // Wildcard
            TokenKind::Identifier(ref name) if name == "_" => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Wildcard,
                    span: start.merge(&self.current_span()),
                })
            }

            // Literal patterns
            TokenKind::IntLiteral(n) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Int(n)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::FloatLiteral(n) => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Float(n)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::StringLiteral(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::String(s)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Bool(true)),
                    span: start.merge(&self.current_span()),
                })
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern {
                    kind: PatternKind::Literal(Literal::Bool(false)),
                    span: start.merge(&self.current_span()),
                })
            }

            // Tuple pattern
            TokenKind::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                while !self.at(&TokenKind::RParen) && !self.is_at_end() {
                    patterns.push(self.parse_pattern()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect_exact(TokenKind::RParen)?;
                Ok(Pattern {
                    kind: PatternKind::Tuple(patterns),
                    span: start.merge(&self.current_span()),
                })
            }

            // Identifier, enum variant, or struct pattern
            TokenKind::Identifier(_) | TokenKind::OptionType | TokenKind::ResultType => {
                let name = self.expect_name()?;
                let mut path = vec![name];

                while self.eat(&TokenKind::Dot) {
                    path.push(self.expect_name()?);
                }

                // Enum variant pattern: Path(patterns)
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at(&TokenKind::RParen) && !self.is_at_end() {
                        fields.push(self.parse_pattern()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect_exact(TokenKind::RParen)?;
                    return Ok(Pattern {
                        kind: PatternKind::EnumVariant { path, fields },
                        span: start.merge(&self.current_span()),
                    });
                }

                // Struct pattern: Path { fields }
                if self.at(&TokenKind::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at(&TokenKind::RBrace) && !self.is_at_end() {
                        let fstart = self.current_span();
                        let fname = self.expect_identifier()?;
                        let fpat = if self.eat(&TokenKind::Colon) {
                            Some(self.parse_pattern()?)
                        } else {
                            None
                        };
                        fields.push(FieldPattern {
                            name: fname,
                            pattern: fpat,
                            span: fstart.merge(&self.current_span()),
                        });
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect_exact(TokenKind::RBrace)?;
                    return Ok(Pattern {
                        kind: PatternKind::Struct {
                            name: path,
                            fields,
                        },
                        span: start.merge(&self.current_span()),
                    });
                }

                // Simple identifier pattern
                if path.len() == 1 {
                    Ok(Pattern {
                        kind: PatternKind::Identifier(path.into_iter().next().unwrap()),
                        span: start.merge(&self.current_span()),
                    })
                } else {
                    // This looks like a path enum variant without args
                    Ok(Pattern {
                        kind: PatternKind::EnumVariant {
                            path,
                            fields: Vec::new(),
                        },
                        span: start.merge(&self.current_span()),
                    })
                }
            }

            _ => Err(CompileError::new(
                "E0104",
                format!("expected pattern, found '{}'", self.peek()),
                start,
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn is_type_keyword(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Int8
            | TokenKind::Int16
            | TokenKind::Int32
            | TokenKind::Int64
            | TokenKind::UInt8
            | TokenKind::UInt16
            | TokenKind::UInt32
            | TokenKind::UInt64
            | TokenKind::Float16
            | TokenKind::Float32
            | TokenKind::Float64
            | TokenKind::BoolType
            | TokenKind::CharType
            | TokenKind::StringType
            | TokenKind::TensorType
            | TokenKind::VecType
            | TokenKind::HashMapType
            | TokenKind::HashSetType
            | TokenKind::OptionType
            | TokenKind::ResultType
            | TokenKind::MutexType
            | TokenKind::RwLockType
            | TokenKind::ChannelType
            | TokenKind::ArcType
    )
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> (Program, Vec<CompileError>) {
        let mut lexer = Lexer::new(src, "test.axon");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, "test.axon");
        let program = parser.parse_program();
        let mut errors = lexer.errors;
        errors.extend(parser.errors);
        (program, errors)
    }

    fn parse_ok(src: &str) -> Program {
        let (prog, errors) = parse(src);
        if !errors.is_empty() {
            for e in &errors {
                eprintln!("{}", e.format_human());
            }
            panic!("expected no errors, got {}", errors.len());
        }
        prog
    }

    // ── Example 1: Hello World ───────────────────────────────

    #[test]
    fn test_example1_hello_world() {
        let prog = parse_ok(r#"
            fn main() {
                println("Hello, Axon!");
            }
        "#);
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0].kind {
            ItemKind::Function(f) => {
                assert_eq!(f.name, "main");
                assert_eq!(f.params.len(), 0);
                assert!(f.return_type.is_none());
            }
            _ => panic!("expected function"),
        }
    }

    // ── Example 2: Tensor Operations ─────────────────────────

    #[test]
    fn test_example2_tensor_operations() {
        let prog = parse_ok(r#"
            fn matrix_multiply_example() {
                val A: Tensor<Float32, [128, 256]> = randn([128, 256]);
                val B: Tensor<Float32, [256, 512]> = randn([256, 512]);
                val C = A @ B;
            }
        "#);
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0].kind {
            ItemKind::Function(f) => {
                assert_eq!(f.name, "matrix_multiply_example");
                let body = f.body.as_ref().unwrap();
                assert_eq!(body.stmts.len(), 3);
            }
            _ => panic!("expected function"),
        }
    }

    // ── Example 3: Ownership and Borrowing ───────────────────

    #[test]
    fn test_example3_ownership() {
        let prog = parse_ok(r#"
            fn process_tensor(data: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
                val normalized = data / 255.0;
                val result = some_model(normalized);
                return result;
            }

            fn main() {
                val input = load_data("input.npy");
                val output = process_tensor(input);
                println("{}", output);
            }
        "#);
        assert_eq!(prog.items.len(), 2);
    }

    // ── Example 4: Neural Network Definition ─────────────────

    #[test]
    fn test_example4_neural_network() {
        let prog = parse_ok(r#"
            model NeuralNet {
                fc1: Linear<784, 256>,
                fc2: Linear<256, 128>,
                fc3: Linear<128, 10>,
            }

            extend NeuralNet {
                fn forward(&self, x: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
                    val h1 = relu(self.fc1.forward(x));
                    val h2 = relu(self.fc2.forward(h1));
                    val output = self.fc3.forward(h2);
                    return output;
                }
            }

            fn main() {
                val net = NeuralNet.new();
                val input = randn([32, 784]);
                val output = net.forward(input);
                println("Output shape: {}", output.shape);
            }
        "#);
        assert_eq!(prog.items.len(), 3);
    }

    // ── Example 5: Training Loop ─────────────────────────────

    #[test]
    fn test_example5_training_loop() {
        let prog = parse_ok(r#"
            fn train_epoch(net: &mut NeuralNet, data: DataLoader, optimizer: &mut Adam): Float32 {
                var total_loss = 0.0;

                for batch in data {
                    val (inputs, targets) = batch;
                    val predictions = net.forward(inputs);
                    val loss = cross_entropy(predictions, targets);
                    total_loss += loss.item();
                    loss.backward();
                    optimizer.step();
                    optimizer.zero_grad();
                }

                return total_loss / data.len() as Float32;
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── Example 6: Device Management ─────────────────────────

    #[test]
    fn test_example6_device_management() {
        let prog = parse_ok(r#"
            fn gpu_example() {
                val cpu_tensor = randn([1024, 1024]);
                val gpu_tensor = cpu_tensor.to_gpu();
                val result = gpu_tensor @ gpu_tensor;
                val cpu_result = result.to_cpu();
                println("{}", cpu_result);
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── Example 7: Error Handling ────────────────────────────

    #[test]
    fn test_example7_error_handling() {
        let prog = parse_ok(r#"
            fn load_model(path: String): Result<NeuralNet, IOError> {
                val file = File.open(path)?;
                val net = deserialize(file)?;
                return Ok(net);
            }

            fn main() {
                match load_model("model.axon") {
                    Ok(net) => {
                        println("Model loaded successfully");
                    },
                    Err(e) => {
                        println("Failed to load model: {}", e);
                    },
                }
            }
        "#);
        assert_eq!(prog.items.len(), 2);
    }

    // ── Example 8: Generics with Trait Bounds ────────────────

    #[test]
    fn test_example8_generics() {
        let prog = parse_ok(r#"
            fn squared_sum<T: Numeric>(values: Tensor<T, [?]>): T {
                val squared = values * values;
                return squared.sum();
            }

            fn main() {
                val int_values: Tensor<Int32, [10]> = arange(0, 10);
                val int_result = squared_sum(int_values);
                val float_values: Tensor<Float32, [10]> = randn([10]);
                val float_result = squared_sum(float_values);
            }
        "#);
        assert_eq!(prog.items.len(), 2);
    }

    // ── Struct with generics ─────────────────────────────────

    #[test]
    fn test_struct_generics() {
        let prog = parse_ok(r#"
            model Pair<A, B> {
                first: A,
                second: B,
            }
        "#);
        match &prog.items[0].kind {
            ItemKind::Struct(s) => {
                assert_eq!(s.name, "Pair");
                assert_eq!(s.generics.len(), 2);
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("expected struct"),
        }
    }

    // ── Enum ─────────────────────────────────────────────────

    #[test]
    fn test_enum() {
        let prog = parse_ok(r#"
            enum Shape {
                Circle(Float64),
                Rectangle(Float64, Float64),
                Point,
            }
        "#);
        match &prog.items[0].kind {
            ItemKind::Enum(e) => {
                assert_eq!(e.name, "Shape");
                assert_eq!(e.variants.len(), 3);
            }
            _ => panic!("expected enum"),
        }
    }

    // ── Trait ────────────────────────────────────────────────

    #[test]
    fn test_trait() {
        let prog = parse_ok(r#"
            trait Drawable {
                fn draw(&self);
                fn area(&self): Float64;
            }
        "#);
        match &prog.items[0].kind {
            ItemKind::Trait(t) => {
                assert_eq!(t.name, "Drawable");
                assert_eq!(t.items.len(), 2);
            }
            _ => panic!("expected trait"),
        }
    }

    // ── Type alias ───────────────────────────────────────────

    #[test]
    fn test_type_alias() {
        let prog = parse_ok("type Matrix = Tensor<Float32, [?, ?]>;");
        match &prog.items[0].kind {
            ItemKind::TypeAlias(ta) => {
                assert_eq!(ta.name, "Matrix");
            }
            _ => panic!("expected type alias"),
        }
    }

    // ── Use declaration ──────────────────────────────────────

    #[test]
    fn test_use_decl() {
        let prog = parse_ok("use std.io.File;");
        match &prog.items[0].kind {
            ItemKind::Use(u) => {
                assert_eq!(u.path, vec!["std", "io", "File"]);
            }
            _ => panic!("expected use"),
        }
    }

    // ── Module declaration ───────────────────────────────────

    #[test]
    fn test_module_decl() {
        let prog = parse_ok("mod utils;");
        match &prog.items[0].kind {
            ItemKind::Module(m) => {
                assert_eq!(m.name, "utils");
                assert!(m.items.is_none());
            }
            _ => panic!("expected module"),
        }
    }

    // ── Operator precedence ──────────────────────────────────

    #[test]
    fn test_matmul_precedence() {
        // @ should be same precedence as *, so `a + b @ c` = `a + (b @ c)`
        let prog = parse_ok("fn main() { val x = a + b @ c; }");
        match &prog.items[0].kind {
            ItemKind::Function(f) => {
                let body = f.body.as_ref().unwrap();
                match &body.stmts[0].kind {
                    StmtKind::Let { initializer: Some(expr), .. } => {
                        match &expr.kind {
                            ExprKind::BinaryOp { op, .. } => {
                                assert_eq!(*op, BinOp::Add, "top-level should be Add");
                            }
                            _ => panic!("expected binary op"),
                        }
                    }
                    _ => panic!("expected let"),
                }
            }
            _ => panic!("expected function"),
        }
    }

    // ── If/Else expression ───────────────────────────────────

    #[test]
    fn test_if_else() {
        let prog = parse_ok(r#"
            fn main() {
                if x > 0 {
                    println("positive");
                } else {
                    println("non-positive");
                }
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── Match expression ─────────────────────────────────────

    #[test]
    fn test_match() {
        let prog = parse_ok(r#"
            fn main() {
                match x {
                    0 => println("zero"),
                    1 => println("one"),
                    _ => println("other"),
                }
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── While loop ───────────────────────────────────────────

    #[test]
    fn test_while_loop() {
        let prog = parse_ok(r#"
            fn main() {
                var i = 0;
                while i < 10 {
                    i += 1;
                }
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── For loop ─────────────────────────────────────────────

    #[test]
    fn test_for_loop() {
        let prog = parse_ok(r#"
            fn main() {
                for item in collection {
                    println("{}", item);
                }
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── Error propagation ────────────────────────────────────

    #[test]
    fn test_error_propagation() {
        let prog = parse_ok(r#"
            fn read_file(path: String): Result<String, IOError> {
                val file = File.open(path)?;
                return Ok(file.read_all()?);
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    // ── Attributes ───────────────────────────────────────────

    #[test]
    fn test_gpu_attribute() {
        let prog = parse_ok(r#"
            @gpu
            fn compute() {
                val x = randn([1024, 1024]);
            }
        "#);
        assert_eq!(prog.items[0].attributes.len(), 1);
        assert!(matches!(prog.items[0].attributes[0], Attribute::Gpu));
    }

    // ── Error messages ───────────────────────────────────────

    #[test]
    fn test_error_missing_semicolon() {
        let (_, errors) = parse("fn main() { val x = 10 }");
        assert!(!errors.is_empty());
        // Should have an error about missing semicolon
        assert!(errors.iter().any(|e| e.message.contains("expected")));
    }

    #[test]
    fn test_error_missing_closing_brace() {
        let (_, errors) = parse("fn main() { val x = 10;");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_multiple_errors_reported() {
        // Multiple errors should be collected
        let (_, errors) = parse(r#"
            fn main() {
                val x = ;
                val y = ;
            }
        "#);
        assert!(errors.len() >= 1, "Should report at least one error, got {}", errors.len());
    }

    // ── Visibility ───────────────────────────────────────────

    #[test]
    fn test_pub_function() {
        let prog = parse_ok("pub fn add(a: Int32, b: Int32): Int32 { return a + b; }");
        assert_eq!(prog.items[0].visibility, Visibility::Public);
    }

    // ── Tensor type ──────────────────────────────────────────

    #[test]
    fn test_tensor_type_parsing() {
        let prog = parse_ok(r#"
            fn process(t: Tensor<Float32, [128, 256]>): Tensor<Float64, [?, 10]> {
                return t;
            }
        "#);
        match &prog.items[0].kind {
            ItemKind::Function(f) => {
                assert_eq!(f.params.len(), 1);
                assert!(f.return_type.is_some());
            }
            _ => panic!("expected function"),
        }
    }

    // ── Unsafe ───────────────────────────────────────────────

    #[test]
    fn test_unsafe_function() {
        let prog = parse_ok("unsafe fn danger() { }");
        // unsafe is parsed as a keyword but we allow it before fn
        assert_eq!(prog.items.len(), 1);
    }

    // ── Complex expressions ──────────────────────────────────

    #[test]
    fn test_method_chain() {
        let prog = parse_ok(r#"
            fn main() {
                val x = a.b().c.d();
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_nested_function_calls() {
        let prog = parse_ok(r#"
            fn main() {
                val x = foo(bar(baz(1, 2), 3), 4);
            }
        "#);
        assert_eq!(prog.items.len(), 1);
    }
}
