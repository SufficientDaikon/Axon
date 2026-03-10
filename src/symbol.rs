// symbol.rs — Symbol table and name resolution (Phase 3b)

use serde::Serialize;
use std::collections::HashMap;

use crate::ast::*;
use crate::error::CompileError;
use crate::span::Span;
use crate::types::*;

// ---------------------------------------------------------------------------
// Scope and symbol kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ScopeKind {
    Module,
    Function,
    Block,
    Impl,
    Trait,
    Loop,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    StructDef,
    EnumDef,
    TraitDef,
    TypeAlias,
    Module,
    Field,
    Method,
    GenericParam,
    EnumVariant,
}

// ---------------------------------------------------------------------------
// SymbolInfo
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub ty: TypeId,
    pub kind: SymbolKind,
    pub mutable: bool,
    pub span: Span,
    pub scope: ScopeId,
    pub visible: bool,
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub bindings: HashMap<String, SymbolId>,
    pub kind: ScopeKind,
}

// ---------------------------------------------------------------------------
// SymbolTable
// ---------------------------------------------------------------------------

pub struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: Vec<SymbolInfo>,
    current_scope: ScopeId,
}

impl SymbolTable {
    pub fn new() -> Self {
        let root = Scope {
            parent: None,
            bindings: HashMap::new(),
            kind: ScopeKind::Module,
        };
        SymbolTable {
            scopes: vec![root],
            symbols: Vec::new(),
            current_scope: ScopeId(0),
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(Scope {
            parent: Some(self.current_scope),
            bindings: HashMap::new(),
            kind,
        });
        self.current_scope = id;
        id
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope.0 as usize].parent {
            self.current_scope = parent;
        }
    }

    pub fn current_scope(&self) -> ScopeId {
        self.current_scope
    }

    /// Define a symbol in the current scope. Returns E1002 on duplicate.
    pub fn define(
        &mut self,
        name: String,
        info: SymbolInfo,
    ) -> Result<SymbolId, CompileError> {
        let scope = &self.scopes[self.current_scope.0 as usize];
        if scope.bindings.contains_key(&name) {
            return Err(CompileError::new(
                "E1002",
                format!("duplicate definition of `{}` in the same scope", name),
                info.span.clone(),
            )
            .with_suggestion(format!(
                "a previous definition of `{}` exists in this scope; consider renaming one of them",
                name
            )));
        }
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(info);
        self.scopes[self.current_scope.0 as usize]
            .bindings
            .insert(name, id);
        Ok(id)
    }

    /// Look up a name by walking up the scope chain from the current scope.
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        let mut scope_id = Some(self.current_scope);
        while let Some(sid) = scope_id {
            if let Some(&sym) = self.scopes[sid.0 as usize].bindings.get(name) {
                return Some(sym);
            }
            scope_id = self.scopes[sid.0 as usize].parent;
        }
        None
    }

    /// Look up a name in a specific scope only (no parent walk).
    pub fn lookup_in(&self, scope: ScopeId, name: &str) -> Option<SymbolId> {
        self.scopes[scope.0 as usize].bindings.get(name).copied()
    }

    pub fn get_symbol(&self, id: SymbolId) -> &SymbolInfo {
        &self.symbols[id.0 as usize]
    }

    pub fn get_symbol_mut(&mut self, id: SymbolId) -> &mut SymbolInfo {
        &mut self.symbols[id.0 as usize]
    }

    pub fn get_scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }

    /// Find a similar name in all visible scopes (for "did you mean?" suggestions).
    fn find_similar(&self, name: &str) -> Option<String> {
        let mut scope_id = Some(self.current_scope);
        let mut best: Option<(usize, String)> = None;
        while let Some(sid) = scope_id {
            for key in self.scopes[sid.0 as usize].bindings.keys() {
                let dist = edit_distance(name, key);
                if dist <= 2 && dist < name.len() {
                    if best.as_ref().map_or(true, |(d, _)| dist < *d) {
                        best = Some((dist, key.clone()));
                    }
                }
            }
            scope_id = self.scopes[sid.0 as usize].parent;
        }
        best.map(|(_, s)| s)
    }
}

/// Simple Levenshtein distance for suggestion generation.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

// ---------------------------------------------------------------------------
// Helper: primitive name → PrimKind
// ---------------------------------------------------------------------------

pub fn prim_from_name(name: &str) -> Option<PrimKind> {
    match name {
        "Int8" | "i8" => Some(PrimKind::Int8),
        "Int16" | "i16" => Some(PrimKind::Int16),
        "Int32" | "i32" => Some(PrimKind::Int32),
        "Int64" | "i64" => Some(PrimKind::Int64),
        "UInt8" | "u8" => Some(PrimKind::UInt8),
        "UInt16" | "u16" => Some(PrimKind::UInt16),
        "UInt32" | "u32" => Some(PrimKind::UInt32),
        "UInt64" | "u64" => Some(PrimKind::UInt64),
        "Float16" | "f16" => Some(PrimKind::Float16),
        "Float32" | "f32" => Some(PrimKind::Float32),
        "Float64" | "f64" => Some(PrimKind::Float64),
        "Bool" | "bool" => Some(PrimKind::Bool),
        "Char" | "char" => Some(PrimKind::Char),
        "String" | "string" => Some(PrimKind::String),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// NameResolver — three-phase resolution over the AST
// ---------------------------------------------------------------------------

pub struct NameResolver<'a> {
    symbols: &'a mut SymbolTable,
    interner: &'a mut TypeInterner,
    errors: Vec<CompileError>,
    next_type_var: u32,
}

impl<'a> NameResolver<'a> {
    pub fn new(symbols: &'a mut SymbolTable, interner: &'a mut TypeInterner) -> Self {
        NameResolver {
            symbols,
            interner,
            errors: Vec::new(),
            next_type_var: 0,
        }
    }

    fn fresh_type_var(&mut self) -> TypeId {
        let id = self.next_type_var;
        self.next_type_var += 1;
        self.interner.intern(Type::TypeVar(id))
    }

    // ── Public entry point ─────────────────────────────────────

    pub fn resolve(&mut self, program: &Program) -> Vec<CompileError> {
        // Phase 1: collect top-level names (forward declarations)
        self.collect_items(&program.items);
        // Phase 2: resolve use/import declarations
        self.resolve_uses(&program.items);
        // Phase 3: resolve all bodies
        self.resolve_items(&program.items);

        std::mem::take(&mut self.errors)
    }

    // ── Phase 1: collect top-level item names ──────────────────

    fn collect_items(&mut self, items: &[Item]) {
        for item in items {
            self.collect_item(item);
        }
    }

    fn collect_item(&mut self, item: &Item) {
        let visible = item.visibility == Visibility::Public;
        match &item.kind {
            ItemKind::Function(decl) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::Function,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
            }
            ItemKind::Struct(decl) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::StructDef,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
            }
            ItemKind::Enum(decl) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::EnumDef,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
            }
            ItemKind::Trait(decl) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::TraitDef,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
            }
            ItemKind::TypeAlias(decl) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::TypeAlias,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
            }
            ItemKind::Module(decl) => {
                let ty = TypeId::UNIT;
                let info = SymbolInfo {
                    name: decl.name.clone(),
                    ty,
                    kind: SymbolKind::Module,
                    mutable: false,
                    span: decl.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible,
                };
                if let Err(e) = self.symbols.define(decl.name.clone(), info) {
                    self.errors.push(e);
                }
                if let Some(ref items) = decl.items {
                    self.symbols.push_scope(ScopeKind::Module);
                    self.collect_items(items);
                    self.symbols.pop_scope();
                }
            }
            ItemKind::Impl(_) | ItemKind::Use(_) => {}
        }
    }

    // ── Phase 2: resolve use/import declarations ───────────────

    fn resolve_uses(&mut self, items: &[Item]) {
        for item in items {
            if let ItemKind::Use(decl) = &item.kind {
                self.resolve_use(decl);
            }
        }
    }

    fn resolve_use(&mut self, decl: &UseDecl) {
        if decl.path.is_empty() {
            return;
        }
        let target_name = decl
            .alias
            .as_ref()
            .unwrap_or_else(|| decl.path.last().unwrap());
        // Try to find the symbol in the root scope first
        let found = self.symbols.lookup_in(ScopeId(0), decl.path.last().unwrap());
        if let Some(sym_id) = found {
            let sym = self.symbols.get_symbol(sym_id);
            if !sym.visible && decl.path.len() > 1 {
                self.errors.push(CompileError::new(
                    "E1004",
                    format!("item `{}` is private", decl.path.last().unwrap()),
                    decl.span.clone(),
                ));
                return;
            }
            // Bind alias in current scope
            let info = SymbolInfo {
                name: target_name.clone(),
                ty: sym.ty,
                kind: sym.kind.clone(),
                mutable: false,
                span: decl.span.clone(),
                scope: self.symbols.current_scope(),
                visible: false,
            };
            if let Err(e) = self.symbols.define(target_name.clone(), info) {
                self.errors.push(e);
            }
        }
        // Uses of external modules are allowed to silently pass (not yet linked)
    }

    // ── Phase 3: resolve all bodies ────────────────────────────

    fn resolve_items(&mut self, items: &[Item]) {
        for item in items {
            self.resolve_item(item);
        }
    }

    fn resolve_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function(decl) => self.resolve_fn(decl),
            ItemKind::Struct(decl) => self.resolve_struct(decl),
            ItemKind::Enum(decl) => self.resolve_enum(decl),
            ItemKind::Impl(block) => self.resolve_impl(block),
            ItemKind::Trait(decl) => self.resolve_trait(decl),
            ItemKind::TypeAlias(decl) => {
                let _ty = self.resolve_type_expr(&decl.ty);
            }
            ItemKind::Module(decl) => {
                if let Some(ref items) = decl.items {
                    self.symbols.push_scope(ScopeKind::Module);
                    self.collect_items(items);
                    self.resolve_items(items);
                    self.symbols.pop_scope();
                }
            }
            ItemKind::Use(_) => {} // already handled in phase 2
        }
    }

    fn resolve_fn(&mut self, decl: &FnDecl) {
        self.symbols.push_scope(ScopeKind::Function);

        // Register generic params
        for gp in &decl.generics {
            let ty = self.interner.intern(Type::Generic(GenericVar {
                name: gp.name.clone(),
                id: self.next_type_var,
            }));
            self.next_type_var += 1;
            let info = SymbolInfo {
                name: gp.name.clone(),
                ty,
                kind: SymbolKind::GenericParam,
                mutable: false,
                span: gp.span.clone(),
                scope: self.symbols.current_scope(),
                visible: false,
            };
            if let Err(e) = self.symbols.define(gp.name.clone(), info) {
                self.errors.push(e);
            }
        }

        // Register parameters
        for param in &decl.params {
            match &param.kind {
                FnParamKind::Typed { name, ty, .. } => {
                    let resolved_ty = self.resolve_type_expr(ty);
                    let info = SymbolInfo {
                        name: name.clone(),
                        ty: resolved_ty,
                        kind: SymbolKind::Parameter,
                        mutable: false,
                        span: param.span.clone(),
                        scope: self.symbols.current_scope(),
                        visible: false,
                    };
                    if let Err(e) = self.symbols.define(name.clone(), info) {
                        self.errors.push(e);
                    }
                }
                FnParamKind::SelfOwned | FnParamKind::SelfRef | FnParamKind::SelfMutRef => {
                    let ty = self.fresh_type_var();
                    let info = SymbolInfo {
                        name: "self".to_string(),
                        ty,
                        kind: SymbolKind::Parameter,
                        mutable: matches!(param.kind, FnParamKind::SelfMutRef),
                        span: param.span.clone(),
                        scope: self.symbols.current_scope(),
                        visible: false,
                    };
                    let _ = self.symbols.define("self".to_string(), info);
                }
            }
        }

        // Resolve return type
        if let Some(ref ret_ty) = decl.return_type {
            let _ty = self.resolve_type_expr(ret_ty);
        }

        // Resolve body
        if let Some(ref body) = decl.body {
            self.resolve_block_inner(body);
        }

        self.symbols.pop_scope();
    }

    fn resolve_struct(&mut self, decl: &StructDecl) {
        for field in &decl.fields {
            let _ty = self.resolve_type_expr(&field.ty);
        }
    }

    fn resolve_enum(&mut self, decl: &EnumDecl) {
        for variant in &decl.variants {
            match &variant.fields {
                EnumVariantKind::Unit => {}
                EnumVariantKind::Tuple(types) => {
                    for ty in types {
                        let _t = self.resolve_type_expr(ty);
                    }
                }
                EnumVariantKind::Struct(fields) => {
                    for field in fields {
                        let _t = self.resolve_type_expr(&field.ty);
                    }
                }
            }
        }
    }

    fn resolve_impl(&mut self, block: &ImplBlock) {
        self.symbols.push_scope(ScopeKind::Impl);
        for gp in &block.generics {
            let ty = self.interner.intern(Type::Generic(GenericVar {
                name: gp.name.clone(),
                id: self.next_type_var,
            }));
            self.next_type_var += 1;
            let info = SymbolInfo {
                name: gp.name.clone(),
                ty,
                kind: SymbolKind::GenericParam,
                mutable: false,
                span: gp.span.clone(),
                scope: self.symbols.current_scope(),
                visible: false,
            };
            let _ = self.symbols.define(gp.name.clone(), info);
        }
        for item in &block.items {
            self.resolve_item(item);
        }
        self.symbols.pop_scope();
    }

    fn resolve_trait(&mut self, decl: &TraitDecl) {
        self.symbols.push_scope(ScopeKind::Trait);
        for gp in &decl.generics {
            let ty = self.interner.intern(Type::Generic(GenericVar {
                name: gp.name.clone(),
                id: self.next_type_var,
            }));
            self.next_type_var += 1;
            let info = SymbolInfo {
                name: gp.name.clone(),
                ty,
                kind: SymbolKind::GenericParam,
                mutable: false,
                span: gp.span.clone(),
                scope: self.symbols.current_scope(),
                visible: false,
            };
            let _ = self.symbols.define(gp.name.clone(), info);
        }
        for item in &decl.items {
            self.resolve_item(item);
        }
        self.symbols.pop_scope();
    }

    fn resolve_block(&mut self, block: &Block) {
        self.symbols.push_scope(ScopeKind::Block);
        self.resolve_block_inner(block);
        self.symbols.pop_scope();
    }

    /// Resolve block contents without creating a new scope (caller manages scope).
    fn resolve_block_inner(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
        if let Some(ref tail) = block.tail_expr {
            self.resolve_expr(tail);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let {
                name,
                mutable,
                ty,
                initializer,
            } => {
                // Resolve initializer first (before binding the name)
                if let Some(ref init) = initializer {
                    self.resolve_expr(init);
                }
                let _resolved_ty = if let Some(ref ty_expr) = ty {
                    self.resolve_type_expr(ty_expr)
                } else {
                    self.fresh_type_var()
                };
                self.resolve_pattern(name, *mutable);
            }
            StmtKind::Expr(expr) => {
                self.resolve_expr(expr);
            }
            StmtKind::Return(opt_expr) => {
                if let Some(ref expr) = opt_expr {
                    self.resolve_expr(expr);
                }
            }
            StmtKind::While { condition, body } => {
                self.resolve_expr(condition);
                self.symbols.push_scope(ScopeKind::Loop);
                self.resolve_block_inner(body);
                self.symbols.pop_scope();
            }
            StmtKind::For {
                pattern,
                iterator,
                body,
            } => {
                self.resolve_expr(iterator);
                self.symbols.push_scope(ScopeKind::Loop);
                self.resolve_pattern(pattern, false);
                self.resolve_block_inner(body);
                self.symbols.pop_scope();
            }
            StmtKind::Item(item) => {
                self.collect_item(item);
                self.resolve_item(item);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(_) => {}
            ExprKind::Identifier(name) => {
                if self.symbols.lookup(name).is_none() {
                    let mut err = CompileError::new(
                        "E1001",
                        format!("undefined name `{}`", name),
                        expr.span.clone(),
                    );
                    if let Some(suggestion) = self.symbols.find_similar(name) {
                        err = err.with_suggestion(format!("did you mean `{}`?", suggestion));
                    }
                    self.errors.push(err);
                }
            }
            ExprKind::Path(segments) => {
                if let Some(first) = segments.first() {
                    if self.symbols.lookup(first).is_none() {
                        let mut err = CompileError::new(
                            "E1001",
                            format!("undefined name `{}`", first),
                            expr.span.clone(),
                        );
                        if let Some(suggestion) = self.symbols.find_similar(first) {
                            err = err.with_suggestion(format!("did you mean `{}`?", suggestion));
                        }
                        self.errors.push(err);
                    }
                }
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::UnaryOp { operand, .. } => {
                self.resolve_expr(operand);
            }
            ExprKind::FnCall { function, args } => {
                self.resolve_expr(function);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::MethodCall {
                receiver, args, ..
            } => {
                self.resolve_expr(receiver);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => {
                self.resolve_expr(object);
            }
            ExprKind::Index { object, index } => {
                self.resolve_expr(object);
                self.resolve_expr(index);
            }
            ExprKind::Slice {
                object,
                start,
                end,
            } => {
                self.resolve_expr(object);
                if let Some(ref s) = start {
                    self.resolve_expr(s);
                }
                if let Some(ref e) = end {
                    self.resolve_expr(e);
                }
            }
            ExprKind::IfElse {
                condition,
                then_block,
                else_block,
            } => {
                self.resolve_expr(condition);
                self.resolve_block(then_block);
                if let Some(ref else_clause) = else_block {
                    match else_clause {
                        ElseClause::ElseBlock(block) => self.resolve_block(block),
                        ElseClause::ElseIf(expr) => self.resolve_expr(expr),
                    }
                }
            }
            ExprKind::Match { expr: scrutinee, arms } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    self.symbols.push_scope(ScopeKind::Block);
                    self.resolve_pattern(&arm.pattern, false);
                    if let Some(ref guard) = arm.guard {
                        self.resolve_expr(guard);
                    }
                    self.resolve_expr(&arm.body);
                    self.symbols.pop_scope();
                }
            }
            ExprKind::Block(block) => {
                self.resolve_block(block);
            }
            ExprKind::Reference { expr: inner, .. } => {
                self.resolve_expr(inner);
            }
            ExprKind::TypeCast {
                expr: inner,
                target_type,
            } => {
                self.resolve_expr(inner);
                let _ty = self.resolve_type_expr(target_type);
            }
            ExprKind::ErrorPropagation(inner) => {
                self.resolve_expr(inner);
            }
            ExprKind::Range { start, end } => {
                if let Some(ref s) = start {
                    self.resolve_expr(s);
                }
                if let Some(ref e) = end {
                    self.resolve_expr(e);
                }
            }
            ExprKind::Assignment { target, value, .. } => {
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            ExprKind::Tuple(elems) => {
                for elem in elems {
                    self.resolve_expr(elem);
                }
            }
            ExprKind::StructLiteral { name, fields } => {
                if let Some(first) = name.first() {
                    if self.symbols.lookup(first).is_none() {
                        self.errors.push(CompileError::new(
                            "E1001",
                            format!("undefined name `{}`", first),
                            expr.span.clone(),
                        ));
                    }
                }
                for field in fields {
                    self.resolve_expr(&field.value);
                }
            }
            ExprKind::Closure {
                params,
                return_type,
                body,
            } => {
                self.symbols.push_scope(ScopeKind::Function);
                for param in params {
                    let ty = if let Some(ref ty_expr) = param.ty {
                        self.resolve_type_expr(ty_expr)
                    } else {
                        self.fresh_type_var()
                    };
                    let info = SymbolInfo {
                        name: param.name.clone(),
                        ty,
                        kind: SymbolKind::Parameter,
                        mutable: false,
                        span: param.span.clone(),
                        scope: self.symbols.current_scope(),
                        visible: false,
                    };
                    if let Err(e) = self.symbols.define(param.name.clone(), info) {
                        self.errors.push(e);
                    }
                }
                if let Some(ref ret_ty) = return_type {
                    let _ty = self.resolve_type_expr(ret_ty);
                }
                self.resolve_expr(body);
                self.symbols.pop_scope();
            }
        }
    }

    fn resolve_pattern(&mut self, pattern: &Pattern, mutable: bool) {
        match &pattern.kind {
            PatternKind::Identifier(name) => {
                let ty = self.fresh_type_var();
                let info = SymbolInfo {
                    name: name.clone(),
                    ty,
                    kind: SymbolKind::Variable,
                    mutable,
                    span: pattern.span.clone(),
                    scope: self.symbols.current_scope(),
                    visible: false,
                };
                if let Err(e) = self.symbols.define(name.clone(), info) {
                    self.errors.push(e);
                }
            }
            PatternKind::Literal(_) => {}
            PatternKind::Wildcard => {}
            PatternKind::Tuple(pats) => {
                for pat in pats {
                    self.resolve_pattern(pat, mutable);
                }
            }
            PatternKind::Struct { name, fields } => {
                if let Some(first) = name.first() {
                    if self.symbols.lookup(first).is_none() {
                        self.errors.push(CompileError::new(
                            "E1001",
                            format!("undefined name `{}`", first),
                            pattern.span.clone(),
                        ));
                    }
                }
                for field in fields {
                    if let Some(ref inner_pat) = field.pattern {
                        self.resolve_pattern(inner_pat, mutable);
                    } else {
                        // Shorthand: `Point { x, y }` — bind x and y
                        let ty = self.fresh_type_var();
                        let info = SymbolInfo {
                            name: field.name.clone(),
                            ty,
                            kind: SymbolKind::Variable,
                            mutable,
                            span: field.span.clone(),
                            scope: self.symbols.current_scope(),
                            visible: false,
                        };
                        if let Err(e) = self.symbols.define(field.name.clone(), info) {
                            self.errors.push(e);
                        }
                    }
                }
            }
            PatternKind::EnumVariant { path, fields } => {
                if let Some(first) = path.first() {
                    if self.symbols.lookup(first).is_none() {
                        self.errors.push(CompileError::new(
                            "E1001",
                            format!("undefined name `{}`", first),
                            pattern.span.clone(),
                        ));
                    }
                }
                for field in fields {
                    self.resolve_pattern(field, mutable);
                }
            }
        }
    }

    pub fn resolve_type_expr(&mut self, ty: &TypeExpr) -> TypeId {
        match &ty.kind {
            TypeExprKind::Named(name) => {
                // Check for primitive types first
                if let Some(prim) = prim_from_name(name) {
                    return self.interner.intern(Type::Primitive(prim));
                }
                // Look up in symbol table
                if let Some(sym_id) = self.symbols.lookup(name) {
                    return self.symbols.get_symbol(sym_id).ty;
                }
                self.errors.push(CompileError::new(
                    "E1001",
                    format!("undefined name `{}`", name),
                    ty.span.clone(),
                ));
                TypeId::ERROR
            }
            TypeExprKind::Path(segments) => {
                if let Some(first) = segments.first() {
                    if let Some(prim) = prim_from_name(first) {
                        if segments.len() == 1 {
                            return self.interner.intern(Type::Primitive(prim));
                        }
                    }
                    if self.symbols.lookup(first).is_none() {
                        self.errors.push(CompileError::new(
                            "E1001",
                            format!("undefined name `{}`", first),
                            ty.span.clone(),
                        ));
                        return TypeId::ERROR;
                    }
                }
                let path: Vec<String> = segments.clone();
                self.interner.intern(Type::Named {
                    path,
                    args: Vec::new(),
                })
            }
            TypeExprKind::Generic { name, args } => {
                // Handle well-known generic types
                match name.as_str() {
                    "Option" => {
                        if let Some(TypeArg::Type(ref inner)) = args.first() {
                            let inner_id = self.resolve_type_expr(inner);
                            return self.interner.intern(Type::Option(inner_id));
                        }
                        self.fresh_type_var()
                    }
                    "Result" => {
                        let ok_id = args
                            .first()
                            .and_then(|a| match a {
                                TypeArg::Type(t) => Some(self.resolve_type_expr(t)),
                                _ => None,
                            })
                            .unwrap_or_else(|| self.fresh_type_var());
                        let err_id = args
                            .get(1)
                            .and_then(|a| match a {
                                TypeArg::Type(t) => Some(self.resolve_type_expr(t)),
                                _ => None,
                            })
                            .unwrap_or_else(|| self.fresh_type_var());
                        self.interner.intern(Type::Result(ok_id, err_id))
                    }
                    "Vec" => {
                        // Represent as a named type with resolved args
                        let resolved_args: Vec<TypeId> = args
                            .iter()
                            .filter_map(|a| match a {
                                TypeArg::Type(t) => Some(self.resolve_type_expr(t)),
                                _ => None,
                            })
                            .collect();
                        self.interner.intern(Type::Named {
                            path: vec!["Vec".to_string()],
                            args: resolved_args,
                        })
                    }
                    _ => {
                        // User-defined generic type
                        if self.symbols.lookup(name).is_none() {
                            if prim_from_name(name).is_none() {
                                self.errors.push(CompileError::new(
                                    "E1001",
                                    format!("undefined name `{}`", name),
                                    ty.span.clone(),
                                ));
                                return TypeId::ERROR;
                            }
                        }
                        let resolved_args: Vec<TypeId> = args
                            .iter()
                            .filter_map(|a| match a {
                                TypeArg::Type(t) => Some(self.resolve_type_expr(t)),
                                _ => None,
                            })
                            .collect();
                        self.interner.intern(Type::Named {
                            path: vec![name.clone()],
                            args: resolved_args,
                        })
                    }
                }
            }
            TypeExprKind::Tensor { dtype, shape } => {
                let dtype_id = self.resolve_type_expr(dtype);
                let resolved_shape: Vec<ShapeDimResolved> = shape
                    .iter()
                    .map(|dim| match dim {
                        ShapeDim::Constant(n) => ShapeDimResolved::Known(*n),
                        ShapeDim::Dynamic => ShapeDimResolved::Dynamic,
                        ShapeDim::Named(name) => ShapeDimResolved::Variable(name.clone()),
                    })
                    .collect();
                self.interner.intern(Type::Tensor(TensorType {
                    dtype: dtype_id,
                    shape: resolved_shape,
                }))
            }
            TypeExprKind::Reference { mutable, inner } => {
                let inner_id = self.resolve_type_expr(inner);
                self.interner.intern(Type::Reference {
                    mutable: *mutable,
                    inner: inner_id,
                })
            }
            TypeExprKind::Function {
                params,
                return_type,
            } => {
                let param_ids: Vec<TypeId> =
                    params.iter().map(|p| self.resolve_type_expr(p)).collect();
                let ret_id = self.resolve_type_expr(return_type);
                self.interner.intern(Type::Function {
                    params: param_ids,
                    ret: ret_id,
                })
            }
            TypeExprKind::Tuple(types) => {
                let ids: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                self.interner.intern(Type::Tuple(ids))
            }
            TypeExprKind::Array { element, size } => {
                let elem_id = self.resolve_type_expr(element);
                self.interner.intern(Type::Array {
                    elem: elem_id,
                    size: *size,
                })
            }
            TypeExprKind::Inferred => self.fresh_type_var(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::types::TypeInterner;

    fn dummy_span() -> Span {
        Span::dummy()
    }

    // ── SymbolTable tests ──────────────────────────────────────

    #[test]
    fn new_symbol_table_has_root_scope() {
        let st = SymbolTable::new();
        assert_eq!(st.current_scope(), ScopeId(0));
        assert_eq!(st.get_scope(ScopeId(0)).kind, ScopeKind::Module);
        assert!(st.get_scope(ScopeId(0)).parent.is_none());
    }

    #[test]
    fn push_and_pop_scope() {
        let mut st = SymbolTable::new();
        let child = st.push_scope(ScopeKind::Function);
        assert_eq!(st.current_scope(), child);
        assert_eq!(st.get_scope(child).parent, Some(ScopeId(0)));

        st.pop_scope();
        assert_eq!(st.current_scope(), ScopeId(0));
    }

    #[test]
    fn define_and_lookup_symbol() {
        let mut st = SymbolTable::new();
        let info = SymbolInfo {
            name: "x".to_string(),
            ty: TypeId::INT32,
            kind: SymbolKind::Variable,
            mutable: true,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let id = st.define("x".to_string(), info).unwrap();
        assert_eq!(st.lookup("x"), Some(id));
        assert_eq!(st.get_symbol(id).name, "x");
        assert_eq!(st.get_symbol(id).ty, TypeId::INT32);
        assert!(st.get_symbol(id).mutable);
    }

    #[test]
    fn duplicate_definition_error() {
        let mut st = SymbolTable::new();
        let info1 = SymbolInfo {
            name: "x".to_string(),
            ty: TypeId::INT32,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        st.define("x".to_string(), info1).unwrap();

        let info2 = SymbolInfo {
            name: "x".to_string(),
            ty: TypeId::BOOL,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let err = st.define("x".to_string(), info2).unwrap_err();
        assert_eq!(err.error_code, "E1002");
        assert!(err.message.contains("duplicate definition"));
    }

    #[test]
    fn scope_chain_lookup_shadows() {
        let mut st = SymbolTable::new();
        let outer = SymbolInfo {
            name: "x".to_string(),
            ty: TypeId::INT32,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let outer_id = st.define("x".to_string(), outer).unwrap();

        st.push_scope(ScopeKind::Block);
        // Inner scope shadows outer
        let inner = SymbolInfo {
            name: "x".to_string(),
            ty: TypeId::BOOL,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: st.current_scope(),
            visible: false,
        };
        let inner_id = st.define("x".to_string(), inner).unwrap();
        assert_ne!(outer_id, inner_id);

        // Lookup from inner scope finds the inner binding
        let found = st.lookup("x").unwrap();
        assert_eq!(found, inner_id);
        assert_eq!(st.get_symbol(found).ty, TypeId::BOOL);

        st.pop_scope();

        // Back in outer scope, finds the outer binding
        let found = st.lookup("x").unwrap();
        assert_eq!(found, outer_id);
        assert_eq!(st.get_symbol(found).ty, TypeId::INT32);
    }

    #[test]
    fn lookup_walks_up_scope_chain() {
        let mut st = SymbolTable::new();
        let info = SymbolInfo {
            name: "top".to_string(),
            ty: TypeId::STRING,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let id = st.define("top".to_string(), info).unwrap();

        st.push_scope(ScopeKind::Function);
        st.push_scope(ScopeKind::Block);

        // Should still find "top" from a deeply nested scope
        assert_eq!(st.lookup("top"), Some(id));
        assert!(st.lookup("nonexistent").is_none());

        st.pop_scope();
        st.pop_scope();
    }

    #[test]
    fn lookup_in_specific_scope() {
        let mut st = SymbolTable::new();
        let info = SymbolInfo {
            name: "a".to_string(),
            ty: TypeId::INT32,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let id = st.define("a".to_string(), info).unwrap();

        st.push_scope(ScopeKind::Block);
        // lookup_in on root scope should find "a"
        assert_eq!(st.lookup_in(ScopeId(0), "a"), Some(id));
        // lookup_in on current (child) scope should NOT find "a"
        assert!(st.lookup_in(st.current_scope(), "a").is_none());

        st.pop_scope();
    }

    #[test]
    fn get_symbol_mut_updates() {
        let mut st = SymbolTable::new();
        let info = SymbolInfo {
            name: "y".to_string(),
            ty: TypeId::INT32,
            kind: SymbolKind::Variable,
            mutable: false,
            span: dummy_span(),
            scope: ScopeId(0),
            visible: false,
        };
        let id = st.define("y".to_string(), info).unwrap();
        assert!(!st.get_symbol(id).mutable);

        st.get_symbol_mut(id).mutable = true;
        assert!(st.get_symbol(id).mutable);
    }

    // ── Name resolution tests ──────────────────────────────────

    #[test]
    fn resolve_simple_program() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();

        // fn add(a: Int32, b: Int32) -> Int32 { a }
        let program = Program {
            items: vec![Item {
                kind: ItemKind::Function(FnDecl {
                    name: "add".to_string(),
                    generics: Vec::new(),
                    params: vec![
                        FnParam {
                            kind: FnParamKind::Typed {
                                name: "a".to_string(),
                                ty: TypeExpr {
                                    kind: TypeExprKind::Named("Int32".to_string()),
                                    span: dummy_span(),
                                },
                                default: None,
                            },
                            span: dummy_span(),
                        },
                        FnParam {
                            kind: FnParamKind::Typed {
                                name: "b".to_string(),
                                ty: TypeExpr {
                                    kind: TypeExprKind::Named("Int32".to_string()),
                                    span: dummy_span(),
                                },
                                default: None,
                            },
                            span: dummy_span(),
                        },
                    ],
                    return_type: Some(TypeExpr {
                        kind: TypeExprKind::Named("Int32".to_string()),
                        span: dummy_span(),
                    }),
                    body: Some(Block {
                        stmts: Vec::new(),
                        tail_expr: Some(Box::new(Expr {
                            kind: ExprKind::Identifier("a".to_string()),
                            span: dummy_span(),
                        })),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                span: dummy_span(),
                visibility: Visibility::Public,
                attributes: Vec::new(),
            }],
            span: dummy_span(),
        };

        let mut resolver = NameResolver::new(&mut st, &mut interner);
        let errors = resolver.resolve(&program);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);

        // "add" should be in the symbol table
        assert!(st.lookup("add").is_some());
    }

    #[test]
    fn resolve_undefined_name_error() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();

        // fn main() { x }
        let program = Program {
            items: vec![Item {
                kind: ItemKind::Function(FnDecl {
                    name: "main".to_string(),
                    generics: Vec::new(),
                    params: Vec::new(),
                    return_type: None,
                    body: Some(Block {
                        stmts: Vec::new(),
                        tail_expr: Some(Box::new(Expr {
                            kind: ExprKind::Identifier("x".to_string()),
                            span: dummy_span(),
                        })),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                span: dummy_span(),
                visibility: Visibility::Private,
                attributes: Vec::new(),
            }],
            span: dummy_span(),
        };

        let mut resolver = NameResolver::new(&mut st, &mut interner);
        let errors = resolver.resolve(&program);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E1001");
        assert!(errors[0].message.contains("undefined name `x`"));
    }

    #[test]
    fn resolve_type_expr_primitives() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();
        let mut resolver = NameResolver::new(&mut st, &mut interner);

        let cases = vec![
            ("Int32", TypeId::INT32),
            ("Bool", TypeId::BOOL),
            ("Float64", TypeId::FLOAT64),
            ("String", TypeId::STRING),
            ("Char", TypeId::CHAR),
        ];
        for (name, expected) in cases {
            let ty_expr = TypeExpr {
                kind: TypeExprKind::Named(name.to_string()),
                span: dummy_span(),
            };
            let id = resolver.resolve_type_expr(&ty_expr);
            assert_eq!(id, expected, "failed for {}", name);
        }
    }

    #[test]
    fn resolve_type_expr_reference() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();
        let mut resolver = NameResolver::new(&mut st, &mut interner);

        let ty_expr = TypeExpr {
            kind: TypeExprKind::Reference {
                mutable: true,
                inner: Box::new(TypeExpr {
                    kind: TypeExprKind::Named("Int32".to_string()),
                    span: dummy_span(),
                }),
            },
            span: dummy_span(),
        };
        let id = resolver.resolve_type_expr(&ty_expr);
        let resolved = interner.resolve(id);
        assert_eq!(
            *resolved,
            Type::Reference {
                mutable: true,
                inner: TypeId::INT32,
            }
        );
    }

    #[test]
    fn resolve_type_expr_tuple() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();
        let mut resolver = NameResolver::new(&mut st, &mut interner);

        let ty_expr = TypeExpr {
            kind: TypeExprKind::Tuple(vec![
                TypeExpr {
                    kind: TypeExprKind::Named("Int32".to_string()),
                    span: dummy_span(),
                },
                TypeExpr {
                    kind: TypeExprKind::Named("Bool".to_string()),
                    span: dummy_span(),
                },
            ]),
            span: dummy_span(),
        };
        let id = resolver.resolve_type_expr(&ty_expr);
        let resolved = interner.resolve(id);
        assert_eq!(*resolved, Type::Tuple(vec![TypeId::INT32, TypeId::BOOL]));
    }

    #[test]
    fn resolve_type_expr_inferred_creates_typevar() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();
        let mut resolver = NameResolver::new(&mut st, &mut interner);

        let ty_expr = TypeExpr {
            kind: TypeExprKind::Inferred,
            span: dummy_span(),
        };
        let id = resolver.resolve_type_expr(&ty_expr);
        let resolved = interner.resolve(id);
        assert!(matches!(resolved, Type::TypeVar(_)));
    }

    #[test]
    fn prim_from_name_coverage() {
        assert_eq!(prim_from_name("Int32"), Some(PrimKind::Int32));
        assert_eq!(prim_from_name("i32"), Some(PrimKind::Int32));
        assert_eq!(prim_from_name("Bool"), Some(PrimKind::Bool));
        assert_eq!(prim_from_name("bool"), Some(PrimKind::Bool));
        assert_eq!(prim_from_name("Float16"), Some(PrimKind::Float16));
        assert_eq!(prim_from_name("f16"), Some(PrimKind::Float16));
        assert_eq!(prim_from_name("NotAType"), None);
    }

    #[test]
    fn resolve_let_with_variable() {
        let mut st = SymbolTable::new();
        let mut interner = TypeInterner::new();

        // fn main() { let x: Int32 = 42; x }
        let program = Program {
            items: vec![Item {
                kind: ItemKind::Function(FnDecl {
                    name: "main".to_string(),
                    generics: Vec::new(),
                    params: Vec::new(),
                    return_type: None,
                    body: Some(Block {
                        stmts: vec![Stmt {
                            kind: StmtKind::Let {
                                name: Pattern {
                                    kind: PatternKind::Identifier("x".to_string()),
                                    span: dummy_span(),
                                },
                                mutable: false,
                                ty: Some(TypeExpr {
                                    kind: TypeExprKind::Named("Int32".to_string()),
                                    span: dummy_span(),
                                }),
                                initializer: Some(Expr {
                                    kind: ExprKind::Literal(Literal::Int(42)),
                                    span: dummy_span(),
                                }),
                            },
                            span: dummy_span(),
                        }],
                        tail_expr: Some(Box::new(Expr {
                            kind: ExprKind::Identifier("x".to_string()),
                            span: dummy_span(),
                        })),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                }),
                span: dummy_span(),
                visibility: Visibility::Private,
                attributes: Vec::new(),
            }],
            span: dummy_span(),
        };

        let mut resolver = NameResolver::new(&mut st, &mut interner);
        let errors = resolver.resolve(&program);
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn edit_distance_works() {
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("abc", "abd"), 1);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
    }
}
