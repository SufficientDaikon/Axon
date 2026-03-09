// typeck.rs — Type checking and inference (Phase 3c)

use std::collections::HashMap;

use crate::ast::*;
use crate::error::CompileError;
use crate::span::Span;
use crate::symbol::*;
use crate::types::*;

// ═══════════════════════════════════════════════════════════════
// Constraint
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
enum Constraint {
    Eq(TypeId, TypeId, Span),
    HasTrait(TypeId, String, Span),
}

// ═══════════════════════════════════════════════════════════════
// TypeChecker
// ═══════════════════════════════════════════════════════════════

pub struct TypeChecker {
    pub interner: TypeInterner,
    pub symbols: SymbolTable,
    pub errors: Vec<CompileError>,
    next_type_var: u32,
    substitutions: HashMap<u32, TypeId>,
    constraints: Vec<Constraint>,
    /// Stack of expected return types for the current function scope.
    return_type_stack: Vec<TypeId>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            interner: TypeInterner::new(),
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            next_type_var: 1000, // high start to avoid collision with NameResolver vars
            substitutions: HashMap::new(),
            constraints: Vec::new(),
            return_type_stack: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == crate::error::Severity::Error)
    }

    // ── Fresh type variable ────────────────────────────────────

    fn fresh_type_var(&mut self) -> TypeId {
        let id = self.next_type_var;
        self.next_type_var += 1;
        self.interner.intern(Type::TypeVar(id))
    }

    // ── Display helper ─────────────────────────────────────────

    fn type_name(&self, id: TypeId) -> String {
        let resolved = self.resolve_type(id);
        format!("{}", self.interner.resolve(resolved))
    }

    // ── Unification (Robinson's algorithm) ─────────────────────

    fn unify(&mut self, a: TypeId, b: TypeId, span: &Span) -> Result<TypeId, CompileError> {
        let a = self.resolve_type(a);
        let b = self.resolve_type(b);

        if a == b {
            return Ok(a);
        }

        // Error type is contagious — absorb silently
        if a == TypeId::ERROR || b == TypeId::ERROR {
            return Ok(TypeId::ERROR);
        }

        // Never type unifies with anything (it's the bottom type)
        if a == TypeId::NEVER {
            return Ok(b);
        }
        if b == TypeId::NEVER {
            return Ok(a);
        }

        let a_ty = self.interner.resolve(a).clone();
        let b_ty = self.interner.resolve(b).clone();

        match (&a_ty, &b_ty) {
            // TypeVar on left
            (Type::TypeVar(var), _) => {
                let var = *var;
                if self.occurs_check(var, b) {
                    return Err(CompileError::new(
                        "E2001",
                        "infinite type detected during unification",
                        span.clone(),
                    ));
                }
                self.substitutions.insert(var, b);
                Ok(b)
            }
            // TypeVar on right
            (_, Type::TypeVar(var)) => {
                let var = *var;
                if self.occurs_check(var, a) {
                    return Err(CompileError::new(
                        "E2001",
                        "infinite type detected during unification",
                        span.clone(),
                    ));
                }
                self.substitutions.insert(var, a);
                Ok(a)
            }
            // Structural: Tuple
            (Type::Tuple(as_), Type::Tuple(bs)) => {
                if as_.len() != bs.len() {
                    return Err(CompileError::new(
                        "E2001",
                        format!(
                            "expected `{}`, found `{}`",
                            self.type_name(a),
                            self.type_name(b)
                        ),
                        span.clone(),
                    ));
                }
                let as_ = as_.clone();
                let bs = bs.clone();
                let mut unified = Vec::new();
                for (ai, bi) in as_.iter().zip(bs.iter()) {
                    unified.push(self.unify(*ai, *bi, span)?);
                }
                Ok(self.interner.intern(Type::Tuple(unified)))
            }
            // Structural: Array
            (Type::Array { elem: e1, size: s1 }, Type::Array { elem: e2, size: s2 }) => {
                if s1 != s2 {
                    return Err(CompileError::new(
                        "E2001",
                        format!(
                            "expected `{}`, found `{}`",
                            self.type_name(a),
                            self.type_name(b)
                        ),
                        span.clone(),
                    ));
                }
                let (e1, e2, s) = (*e1, *e2, *s1);
                let elem = self.unify(e1, e2, span)?;
                Ok(self.interner.intern(Type::Array { elem, size: s }))
            }
            // Structural: Reference
            (
                Type::Reference { mutable: m1, inner: i1 },
                Type::Reference { mutable: m2, inner: i2 },
            ) => {
                let (m1, m2, i1, i2) = (*m1, *m2, *i1, *i2);
                // &mut T can coerce to &T, but not the other way
                if !m1 && m2 {
                    // trying to unify &T with &mut T — ok, demote to &T
                    let inner = self.unify(i1, i2, span)?;
                    Ok(self.interner.intern(Type::Reference { mutable: false, inner }))
                } else if m1 && !m2 {
                    // &mut T unified with &T — demote to &T
                    let inner = self.unify(i1, i2, span)?;
                    Ok(self.interner.intern(Type::Reference { mutable: false, inner }))
                } else {
                    let inner = self.unify(i1, i2, span)?;
                    Ok(self.interner.intern(Type::Reference { mutable: m1, inner }))
                }
            }
            // Structural: Function
            (
                Type::Function { params: p1, ret: r1 },
                Type::Function { params: p2, ret: r2 },
            ) => {
                if p1.len() != p2.len() {
                    return Err(CompileError::new(
                        "E2001",
                        format!(
                            "expected `{}`, found `{}`",
                            self.type_name(a),
                            self.type_name(b)
                        ),
                        span.clone(),
                    ));
                }
                let p1 = p1.clone();
                let p2 = p2.clone();
                let (r1, r2) = (*r1, *r2);
                let mut params = Vec::new();
                for (a_p, b_p) in p1.iter().zip(p2.iter()) {
                    params.push(self.unify(*a_p, *b_p, span)?);
                }
                let ret = self.unify(r1, r2, span)?;
                Ok(self.interner.intern(Type::Function { params, ret }))
            }
            // Structural: Option
            (Type::Option(i1), Type::Option(i2)) => {
                let (i1, i2) = (*i1, *i2);
                let inner = self.unify(i1, i2, span)?;
                Ok(self.interner.intern(Type::Option(inner)))
            }
            // Structural: Result
            (Type::Result(o1, e1), Type::Result(o2, e2)) => {
                let (o1, e1, o2, e2) = (*o1, *e1, *o2, *e2);
                let ok = self.unify(o1, o2, span)?;
                let err = self.unify(e1, e2, span)?;
                Ok(self.interner.intern(Type::Result(ok, err)))
            }
            // Mismatch
            _ => Err(CompileError::new(
                "E2001",
                format!(
                    "expected `{}`, found `{}`",
                    self.type_name(a),
                    self.type_name(b)
                ),
                span.clone(),
            )),
        }
    }

    fn occurs_check(&self, var: u32, ty: TypeId) -> bool {
        let ty = self.resolve_type(ty);
        let resolved = self.interner.resolve(ty);
        match resolved {
            Type::TypeVar(v) => *v == var,
            Type::Tuple(elems) => {
                let elems = elems.clone();
                elems.iter().any(|e| self.occurs_check(var, *e))
            }
            Type::Array { elem, .. } => self.occurs_check(var, *elem),
            Type::Reference { inner, .. } => self.occurs_check(var, *inner),
            Type::Function { params, ret } => {
                let params = params.clone();
                let ret = *ret;
                params.iter().any(|p| self.occurs_check(var, *p))
                    || self.occurs_check(var, ret)
            }
            Type::Option(inner) => self.occurs_check(var, *inner),
            Type::Result(ok, err) => {
                let (ok, err) = (*ok, *err);
                self.occurs_check(var, ok) || self.occurs_check(var, err)
            }
            _ => false,
        }
    }

    // ── Substitution resolution ────────────────────────────────

    fn resolve_type(&self, id: TypeId) -> TypeId {
        let ty = self.interner.resolve(id);
        if let Type::TypeVar(var) = ty {
            let var = *var;
            if let Some(&resolved) = self.substitutions.get(&var) {
                if resolved != id {
                    return self.resolve_type(resolved);
                }
            }
        }
        id
    }

    fn apply_substitutions(&mut self) {
        // Resolve all constraints (currently a no-op beyond unification already done)
        for constraint in std::mem::take(&mut self.constraints) {
            match constraint {
                Constraint::Eq(a, b, ref span) => {
                    if let Err(e) = self.unify(a, b, span) {
                        self.errors.push(e);
                    }
                }
                Constraint::HasTrait(_ty, _trait_name, _span) => {
                    // Trait bound checking is deferred to a later phase
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Program / Item checking
    // ═══════════════════════════════════════════════════════════

    pub fn check_program(&mut self, program: &Program) {
        // First pass: register all function signatures so recursive/mutual calls resolve
        for item in &program.items {
            if let ItemKind::Function(decl) = &item.kind {
                self.register_function_signature(decl);
            }
        }
        // Second pass: check all items (function bodies, struct layouts, etc.)
        for item in &program.items {
            self.check_item(item);
        }
        self.apply_substitutions();
    }

    /// Pre-register a function's parameter types and return type so recursive calls resolve.
    fn register_function_signature(&mut self, decl: &FnDecl) {
        let mut param_types = Vec::new();
        for param in &decl.params {
            match &param.kind {
                FnParamKind::Typed { ty, .. } => {
                    let resolved_ty = self.resolve_type_expr(ty);
                    param_types.push(resolved_ty);
                }
                FnParamKind::SelfOwned | FnParamKind::SelfRef | FnParamKind::SelfMutRef => {
                    param_types.push(self.fresh_type_var());
                }
            }
        }
        let ret_ty = if let Some(ref ret) = decl.return_type {
            self.resolve_type_expr(ret)
        } else {
            TypeId::UNIT
        };
        let fn_ty = self.interner.intern(Type::Function {
            params: param_types,
            ret: ret_ty,
        });
        if let Some(sym_id) = self.symbols.lookup(&decl.name) {
            self.symbols.get_symbol_mut(sym_id).ty = fn_ty;
        }
    }

    fn check_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function(decl) => self.check_function(decl),
            ItemKind::Struct(decl) => self.check_struct(decl),
            ItemKind::Enum(decl) => self.check_enum(decl),
            ItemKind::Impl(block) => self.check_impl(block),
            ItemKind::Trait(decl) => self.check_trait(decl),
            ItemKind::TypeAlias(_) | ItemKind::Module(_) | ItemKind::Use(_) => {}
        }
    }

    fn check_function(&mut self, decl: &FnDecl) {
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
            let _ = self.symbols.define(gp.name.clone(), info);
        }

        // Register parameters
        let mut param_types = Vec::new();
        for param in &decl.params {
            match &param.kind {
                FnParamKind::Typed { name, ty, .. } => {
                    let resolved_ty = self.resolve_type_expr(ty);
                    param_types.push(resolved_ty);
                    let info = SymbolInfo {
                        name: name.clone(),
                        ty: resolved_ty,
                        kind: SymbolKind::Parameter,
                        mutable: false,
                        span: param.span.clone(),
                        scope: self.symbols.current_scope(),
                        visible: false,
                    };
                    let _ = self.symbols.define(name.clone(), info);
                }
                FnParamKind::SelfOwned | FnParamKind::SelfRef | FnParamKind::SelfMutRef => {
                    let ty = self.fresh_type_var();
                    param_types.push(ty);
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

        // Declared return type
        let declared_ret = if let Some(ref ret_ty) = decl.return_type {
            self.resolve_type_expr(ret_ty)
        } else {
            TypeId::UNIT
        };

        // Push expected return type so return statements can be checked
        self.return_type_stack.push(declared_ret);

        // Check body
        if let Some(ref body) = decl.body {
            let body_ty = self.check_block(body);
            // Only unify body type with return type if body has a tail expression
            if body.tail_expr.is_some() {
                if let Err(e) = self.unify(body_ty, declared_ret, &decl.span) {
                    self.errors.push(e);
                }
            }
        }

        self.return_type_stack.pop();

        // Update the function symbol's type
        let fn_ty = self.interner.intern(Type::Function {
            params: param_types,
            ret: declared_ret,
        });
        if let Some(sym_id) = self.symbols.lookup(&decl.name) {
            self.symbols.get_symbol_mut(sym_id).ty = fn_ty;
        }

        self.symbols.pop_scope();
    }

    fn check_struct(&mut self, decl: &StructDecl) {
        let mut field_types = Vec::new();
        for field in &decl.fields {
            let ty = self.resolve_type_expr(&field.ty);
            field_types.push((field.name.clone(), ty));
        }
        let struct_ty = self.interner.intern(Type::Struct {
            name: decl.name.clone(),
            fields: field_types,
            generics: Vec::new(),
        });
        // Update the struct symbol
        if let Some(sym_id) = self.symbols.lookup(&decl.name) {
            self.symbols.get_symbol_mut(sym_id).ty = struct_ty;
        }
    }

    fn check_enum(&mut self, decl: &EnumDecl) {
        let mut variants = Vec::new();
        for variant in &decl.variants {
            let vtype = match &variant.fields {
                EnumVariantKind::Unit => EnumVariantType::Unit,
                EnumVariantKind::Tuple(types) => {
                    let tys: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                    EnumVariantType::Tuple(tys)
                }
                EnumVariantKind::Struct(fields) => {
                    let fs: Vec<(String, TypeId)> = fields
                        .iter()
                        .map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty)))
                        .collect();
                    EnumVariantType::Struct(fs)
                }
            };
            variants.push((variant.name.clone(), vtype));
        }
        let enum_ty = self.interner.intern(Type::Enum {
            name: decl.name.clone(),
            variants,
            generics: Vec::new(),
        });
        if let Some(sym_id) = self.symbols.lookup(&decl.name) {
            self.symbols.get_symbol_mut(sym_id).ty = enum_ty;
        }
    }

    fn check_impl(&mut self, block: &ImplBlock) {
        self.symbols.push_scope(ScopeKind::Impl);
        for item in &block.items {
            self.check_item(item);
        }
        self.symbols.pop_scope();
    }

    fn check_trait(&mut self, decl: &TraitDecl) {
        self.symbols.push_scope(ScopeKind::Trait);
        for item in &decl.items {
            self.check_item(item);
        }
        self.symbols.pop_scope();
    }

    // ═══════════════════════════════════════════════════════════
    // Block / Statement checking
    // ═══════════════════════════════════════════════════════════

    fn check_block(&mut self, block: &Block) -> TypeId {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(ref tail) = block.tail_expr {
            self.check_expr(tail)
        } else {
            TypeId::UNIT
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let {
                name,
                mutable,
                ty,
                initializer,
            } => {
                let annotation_ty = ty.as_ref().map(|t| self.resolve_type_expr(t));
                let init_ty = initializer.as_ref().map(|e| self.check_expr(e));

                let final_ty = match (annotation_ty, init_ty) {
                    (Some(ann), Some(init)) => {
                        match self.unify(ann, init, &stmt.span) {
                            Ok(t) => t,
                            Err(e) => {
                                self.errors.push(e);
                                ann
                            }
                        }
                    }
                    (Some(ann), None) => ann,
                    (None, Some(init)) => init,
                    (None, None) => {
                        self.errors.push(CompileError::new(
                            "E2014",
                            "type annotations needed: cannot infer type",
                            stmt.span.clone(),
                        ));
                        TypeId::ERROR
                    }
                };

                self.check_pattern(&name, final_ty);

                // Define the variable in the type checker's scope
                if let PatternKind::Identifier(ref var_name) = name.kind {
                    let info = SymbolInfo {
                        name: var_name.clone(),
                        ty: final_ty,
                        kind: SymbolKind::Variable,
                        mutable: *mutable,
                        span: stmt.span.clone(),
                        scope: self.symbols.current_scope(),
                        visible: false,
                    };
                    let _ = self.symbols.define(var_name.clone(), info);
                }
            }
            StmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            StmtKind::Return(opt_expr) => {
                let ret_ty = if let Some(ref expr) = opt_expr {
                    self.check_expr(expr)
                } else {
                    TypeId::UNIT
                };
                // Unify with the current function's declared return type
                if let Some(&expected) = self.return_type_stack.last() {
                    if let Err(e) = self.unify(ret_ty, expected, &stmt.span) {
                        self.errors.push(e);
                    }
                }
            }
            StmtKind::While { condition, body } => {
                let cond_ty = self.check_expr(condition);
                if let Err(e) = self.unify(cond_ty, TypeId::BOOL, &condition.span) {
                    self.errors.push(e);
                }
                self.check_block(body);
            }
            StmtKind::For { pattern, iterator, body } => {
                let _iter_ty = self.check_expr(iterator);
                // The pattern type would need iterator element type inference;
                // for now we use a fresh type var
                let elem_ty = self.fresh_type_var();
                self.check_pattern(pattern, elem_ty);
                self.check_block(body);
            }
            StmtKind::Item(item) => {
                self.check_item(item);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Expression checking
    // ═══════════════════════════════════════════════════════════

    fn check_expr(&mut self, expr: &Expr) -> TypeId {
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Int(_) => TypeId::INT64,
                Literal::Float(_) => TypeId::FLOAT64,
                Literal::String(_) => TypeId::STRING,
                Literal::Bool(_) => TypeId::BOOL,
                Literal::Char(_) => TypeId::CHAR,
            },
            ExprKind::Identifier(name) => {
                if let Some(sym_id) = self.symbols.lookup(name) {
                    let ty = self.symbols.get_symbol(sym_id).ty;
                    self.resolve_type(ty)
                } else {
                    // Already reported by NameResolver
                    TypeId::ERROR
                }
            }
            ExprKind::Path(segments) => {
                if let Some(first) = segments.first() {
                    if let Some(sym_id) = self.symbols.lookup(first) {
                        let ty = self.symbols.get_symbol(sym_id).ty;
                        return self.resolve_type(ty);
                    }
                }
                TypeId::ERROR
            }
            ExprKind::BinaryOp { left, op, right } => {
                self.check_binary_op(left, op, right, &expr.span)
            }
            ExprKind::UnaryOp { op, operand } => {
                self.check_unary_op(op, operand, &expr.span)
            }
            ExprKind::FnCall { function, args } => {
                self.check_fn_call(function, args, &expr.span)
            }
            ExprKind::MethodCall { receiver, method, args } => {
                self.check_method_call(receiver, method, args, &expr.span)
            }
            ExprKind::FieldAccess { object, field } => {
                self.check_field_access(object, field, &expr.span)
            }
            ExprKind::Index { object, index } => {
                self.check_index(object, index, &expr.span)
            }
            ExprKind::Slice { object, .. } => {
                let obj_ty = self.check_expr(object);
                // Slicing returns the same collection type for now
                obj_ty
            }
            ExprKind::IfElse { condition, then_block, else_block } => {
                self.check_if_else(condition, then_block, else_block, &expr.span)
            }
            ExprKind::Match { expr: scrutinee, arms } => {
                self.check_match(scrutinee, arms, &expr.span)
            }
            ExprKind::Block(block) => self.check_block(block),
            ExprKind::Reference { mutable, expr: inner } => {
                self.check_reference(*mutable, inner, &expr.span)
            }
            ExprKind::TypeCast { expr: inner, target_type } => {
                self.check_type_cast(inner, target_type, &expr.span)
            }
            ExprKind::ErrorPropagation(inner) => {
                self.check_error_propagation(inner, &expr.span)
            }
            ExprKind::Range { .. } => {
                // Range types are a special built-in; use a named type placeholder
                self.interner.intern(Type::Named {
                    path: vec!["Range".to_string()],
                    args: vec![TypeId::INT64],
                })
            }
            ExprKind::Assignment { target, op, value } => {
                self.check_assignment(target, op, value, &expr.span)
            }
            ExprKind::Tuple(elems) => {
                let tys: Vec<TypeId> = elems.iter().map(|e| self.check_expr(e)).collect();
                self.interner.intern(Type::Tuple(tys))
            }
            ExprKind::StructLiteral { name, fields } => {
                self.check_struct_literal(name, fields, &expr.span)
            }
            ExprKind::Closure { params, return_type, body } => {
                self.check_closure(params, return_type, body, &expr.span)
            }
        }
    }

    fn check_binary_op(&mut self, left: &Expr, op: &BinOp, right: &Expr, span: &Span) -> TypeId {
        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);

        // Error propagation
        if left_ty == TypeId::ERROR || right_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        match op {
            // Arithmetic
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                let left_resolved = self.interner.resolve(self.resolve_type(left_ty)).clone();
                if !left_resolved.is_numeric() {
                    self.errors.push(CompileError::new(
                        "E2002",
                        format!("cannot apply `{:?}` to `{}`", op, self.type_name(left_ty)),
                        span.clone(),
                    ));
                    return TypeId::ERROR;
                }
                match self.unify(left_ty, right_ty, span) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        TypeId::ERROR
                    }
                }
            }
            // MatMul
            BinOp::MatMul => {
                let lt = self.interner.resolve(self.resolve_type(left_ty)).clone();
                let rt = self.interner.resolve(self.resolve_type(right_ty)).clone();
                if !lt.is_tensor() || !rt.is_tensor() {
                    self.errors.push(CompileError::new(
                        "E2002",
                        format!("cannot apply `@` to `{}`", self.type_name(left_ty)),
                        span.clone(),
                    ));
                    return TypeId::ERROR;
                }
                // Return a tensor with fresh shape (shape inference deferred to shapes.rs)
                let dtype = if let Type::Tensor(ref t) = lt { t.dtype } else { TypeId::FLOAT32 };
                self.interner.intern(Type::Tensor(TensorType {
                    dtype,
                    shape: vec![ShapeDimResolved::Dynamic],
                }))
            }
            // Comparison
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                if let Err(e) = self.unify(left_ty, right_ty, span) {
                    self.errors.push(e);
                }
                TypeId::BOOL
            }
            // Logical
            BinOp::And | BinOp::Or => {
                if let Err(e) = self.unify(left_ty, TypeId::BOOL, &left.span) {
                    self.errors.push(e);
                }
                if let Err(e) = self.unify(right_ty, TypeId::BOOL, &right.span) {
                    self.errors.push(e);
                }
                TypeId::BOOL
            }
        }
    }

    fn check_unary_op(&mut self, op: &UnaryOp, operand: &Expr, span: &Span) -> TypeId {
        let operand_ty = self.check_expr(operand);
        if operand_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        match op {
            UnaryOp::Neg => {
                let resolved = self.interner.resolve(self.resolve_type(operand_ty)).clone();
                if !resolved.is_numeric() {
                    self.errors.push(CompileError::new(
                        "E2002",
                        format!("cannot apply `-` to `{}`", self.type_name(operand_ty)),
                        span.clone(),
                    ));
                    return TypeId::ERROR;
                }
                operand_ty
            }
            UnaryOp::Not => {
                if let Err(e) = self.unify(operand_ty, TypeId::BOOL, span) {
                    self.errors.push(e);
                    return TypeId::ERROR;
                }
                TypeId::BOOL
            }
            UnaryOp::Ref => {
                self.interner.intern(Type::Reference {
                    mutable: false,
                    inner: operand_ty,
                })
            }
            UnaryOp::MutRef => {
                self.interner.intern(Type::Reference {
                    mutable: true,
                    inner: operand_ty,
                })
            }
        }
    }

    fn check_fn_call(&mut self, func: &Expr, args: &[Expr], span: &Span) -> TypeId {
        let func_ty = self.check_expr(func);
        if func_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let resolved = self.interner.resolve(self.resolve_type(func_ty)).clone();
        match resolved {
            Type::Function { params, ret } => {
                if params.len() != args.len() {
                    self.errors.push(CompileError::new(
                        "E2003",
                        format!("expected {} arguments, found {}", params.len(), args.len()),
                        span.clone(),
                    ));
                    return TypeId::ERROR;
                }
                for (param_ty, arg_expr) in params.iter().zip(args.iter()) {
                    let arg_ty = self.check_expr(arg_expr);
                    if let Err(e) = self.unify(*param_ty, arg_ty, &arg_expr.span) {
                        self.errors.push(e);
                    }
                }
                ret
            }
            // TypeVar — the callee type is not yet resolved; infer it as a function
            Type::TypeVar(_) => {
                let arg_tys: Vec<TypeId> = args.iter().map(|a| self.check_expr(a)).collect();
                let ret = self.fresh_type_var();
                let fn_ty = self.interner.intern(Type::Function {
                    params: arg_tys,
                    ret,
                });
                if let Err(e) = self.unify(func_ty, fn_ty, span) {
                    self.errors.push(e);
                }
                ret
            }
            _ => {
                self.errors.push(CompileError::new(
                    "E2004",
                    format!("`{}` is not callable", self.type_name(func_ty)),
                    span.clone(),
                ));
                TypeId::ERROR
            }
        }
    }

    fn check_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
        span: &Span,
    ) -> TypeId {
        let recv_ty = self.check_expr(receiver);
        if recv_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let resolved = self.interner.resolve(self.resolve_type(recv_ty)).clone();

        // Check struct methods via trait/impl (simplified: look for method in symbol table)
        match resolved {
            Type::Struct { ref name, .. } | Type::Enum { ref name, .. } => {
                // Try to find method in symbol table (registered by impl blocks)
                let method_name = format!("{}::{}", name, method);
                if let Some(sym_id) = self.symbols.lookup(&method_name) {
                    let method_ty = self.symbols.get_symbol(sym_id).ty;
                    let method_resolved = self.interner.resolve(self.resolve_type(method_ty)).clone();
                    if let Type::Function { params, ret } = method_resolved {
                        // Skip self parameter
                        let non_self_params = if !params.is_empty() { &params[1..] } else { &params };
                        if non_self_params.len() != args.len() {
                            self.errors.push(CompileError::new(
                                "E2003",
                                format!("expected {} arguments, found {}", non_self_params.len(), args.len()),
                                span.clone(),
                            ));
                            return TypeId::ERROR;
                        }
                        for (param_ty, arg_expr) in non_self_params.iter().zip(args.iter()) {
                            let arg_ty = self.check_expr(arg_expr);
                            if let Err(e) = self.unify(*param_ty, arg_ty, &arg_expr.span) {
                                self.errors.push(e);
                            }
                        }
                        return ret;
                    }
                }
                // Method not found — return fresh type var (lenient for now)
                self.fresh_type_var()
            }
            _ => {
                // For other types, return a fresh type var (lenient)
                for arg in args {
                    self.check_expr(arg);
                }
                self.fresh_type_var()
            }
        }
    }

    fn check_if_else(
        &mut self,
        condition: &Expr,
        then_block: &Block,
        else_clause: &Option<ElseClause>,
        span: &Span,
    ) -> TypeId {
        let cond_ty = self.check_expr(condition);
        if let Err(e) = self.unify(cond_ty, TypeId::BOOL, &condition.span) {
            self.errors.push(e);
        }

        let then_ty = self.check_block(then_block);

        match else_clause {
            Some(ElseClause::ElseBlock(else_block)) => {
                let else_ty = self.check_block(else_block);
                match self.unify(then_ty, else_ty, span) {
                    Ok(t) => t,
                    Err(_) => {
                        self.errors.push(CompileError::new(
                            "E2008",
                            "if and else branches have incompatible types",
                            span.clone(),
                        ).with_note(format!(
                            "then branch has type `{}`, else branch has type `{}`",
                            self.type_name(then_ty),
                            self.type_name(else_ty),
                        )));
                        TypeId::ERROR
                    }
                }
            }
            Some(ElseClause::ElseIf(else_if_expr)) => {
                let else_ty = self.check_expr(else_if_expr);
                match self.unify(then_ty, else_ty, span) {
                    Ok(t) => t,
                    Err(_) => {
                        self.errors.push(CompileError::new(
                            "E2008",
                            "if and else branches have incompatible types",
                            span.clone(),
                        ));
                        TypeId::ERROR
                    }
                }
            }
            None => {
                // No else branch — if expression type is Unit
                TypeId::UNIT
            }
        }
    }

    fn check_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        span: &Span,
    ) -> TypeId {
        let scrutinee_ty = self.check_expr(scrutinee);

        if arms.is_empty() {
            return TypeId::UNIT;
        }

        // Check each arm pattern against scrutinee type
        let mut result_ty: Option<TypeId> = None;
        for arm in arms {
            self.check_pattern(&arm.pattern, scrutinee_ty);
            if let Some(ref guard) = arm.guard {
                let guard_ty = self.check_expr(guard);
                if let Err(e) = self.unify(guard_ty, TypeId::BOOL, &guard.span) {
                    self.errors.push(e);
                }
            }
            let arm_ty = self.check_expr(&arm.body);
            match result_ty {
                None => result_ty = Some(arm_ty),
                Some(prev) => {
                    match self.unify(prev, arm_ty, span) {
                        Ok(t) => result_ty = Some(t),
                        Err(_) => {
                            self.errors.push(CompileError::new(
                                "E2009",
                                "match arms have incompatible types",
                                span.clone(),
                            ).with_note(format!(
                                "first arm has type `{}`, but this arm has type `{}`",
                                self.type_name(prev),
                                self.type_name(arm_ty),
                            )));
                            return TypeId::ERROR;
                        }
                    }
                }
            }
        }

        result_ty.unwrap_or(TypeId::UNIT)
    }

    fn check_field_access(&mut self, object: &Expr, field: &str, span: &Span) -> TypeId {
        let obj_ty = self.check_expr(object);
        if obj_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let resolved = self.interner.resolve(self.resolve_type(obj_ty)).clone();
        match resolved {
            Type::Struct { ref name, ref fields, .. } => {
                for (fname, fty) in fields {
                    if fname == field {
                        return *fty;
                    }
                }
                self.errors.push(CompileError::new(
                    "E2006",
                    format!("no field `{}` on type `{}`", field, name),
                    span.clone(),
                ));
                TypeId::ERROR
            }
            Type::Tuple(ref elems) => {
                // Tuple field access by index: t.0, t.1, etc.
                if let Ok(idx) = field.parse::<usize>() {
                    if idx < elems.len() {
                        return elems[idx];
                    }
                }
                self.errors.push(CompileError::new(
                    "E2006",
                    format!("no field `{}` on type `{}`", field, self.type_name(obj_ty)),
                    span.clone(),
                ));
                TypeId::ERROR
            }
            _ => {
                // For unresolved types, return fresh type var (lenient)
                if let Type::TypeVar(_) = resolved {
                    return self.fresh_type_var();
                }
                self.errors.push(CompileError::new(
                    "E2006",
                    format!("no field `{}` on type `{}`", field, self.type_name(obj_ty)),
                    span.clone(),
                ));
                TypeId::ERROR
            }
        }
    }

    fn check_index(&mut self, object: &Expr, index: &Expr, span: &Span) -> TypeId {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);

        if obj_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let resolved = self.interner.resolve(self.resolve_type(obj_ty)).clone();
        match resolved {
            Type::Array { elem, .. } => {
                // Index must be integer
                let idx_resolved = self.interner.resolve(self.resolve_type(idx_ty)).clone();
                if let Type::Primitive(p) = idx_resolved {
                    if !p.is_integer() {
                        self.errors.push(CompileError::new(
                            "E2007",
                            format!("cannot index with `{}`", self.type_name(idx_ty)),
                            span.clone(),
                        ));
                    }
                }
                elem
            }
            Type::Tensor(_) => {
                // Tensor indexing returns a tensor or scalar
                self.fresh_type_var()
            }
            _ => {
                if let Type::TypeVar(_) = resolved {
                    return self.fresh_type_var();
                }
                self.errors.push(CompileError::new(
                    "E2007",
                    format!("cannot index into `{}`", self.type_name(obj_ty)),
                    span.clone(),
                ));
                TypeId::ERROR
            }
        }
    }

    fn check_struct_literal(
        &mut self,
        name: &[String],
        fields: &[StructLiteralField],
        span: &Span,
    ) -> TypeId {
        let struct_name = name.join("::");

        // Look up the struct definition
        if let Some(sym_id) = self.symbols.lookup(&struct_name) {
            let struct_ty_id = self.symbols.get_symbol(sym_id).ty;
            let struct_ty = self.interner.resolve(self.resolve_type(struct_ty_id)).clone();

            if let Type::Struct { ref name, fields: ref def_fields, .. } = struct_ty {
                let name = name.clone();
                let def_fields = def_fields.clone();

                // Check each provided field
                for field in fields {
                    let val_ty = self.check_expr(&field.value);
                    if let Some((_fname, expected_ty)) = def_fields.iter().find(|(n, _)| *n == field.name) {
                        if let Err(e) = self.unify(*expected_ty, val_ty, &field.span) {
                            self.errors.push(e);
                        }
                    } else {
                        self.errors.push(CompileError::new(
                            "E2006",
                            format!("no field `{}` on type `{}`", field.name, name),
                            field.span.clone(),
                        ));
                    }
                }

                // Check for missing required fields
                let provided: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                for (fname, _) in &def_fields {
                    if !provided.contains(&fname.as_str()) {
                        self.errors.push(CompileError::new(
                            "E2005",
                            format!("missing field `{}` in struct `{}`", fname, name),
                            span.clone(),
                        ));
                    }
                }

                return struct_ty_id;
            }
        }

        // Type not found or not a struct; check field exprs anyway
        for field in fields {
            self.check_expr(&field.value);
        }
        TypeId::ERROR
    }

    fn check_type_cast(&mut self, expr: &Expr, target: &TypeExpr, span: &Span) -> TypeId {
        let from_ty = self.check_expr(expr);
        let to_ty = self.resolve_type_expr(target);

        if from_ty == TypeId::ERROR || to_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let from_resolved = self.interner.resolve(self.resolve_type(from_ty)).clone();
        let to_resolved = self.interner.resolve(self.resolve_type(to_ty)).clone();

        // Allow numeric-to-numeric casts
        if from_resolved.is_numeric() && to_resolved.is_numeric() {
            return to_ty;
        }

        // Allow same-type cast
        if self.resolve_type(from_ty) == self.resolve_type(to_ty) {
            return to_ty;
        }

        // Disallow other casts
        self.errors.push(CompileError::new(
            "E2012",
            format!(
                "cannot cast `{}` to `{}`",
                self.type_name(from_ty),
                self.type_name(to_ty)
            ),
            span.clone(),
        ));
        TypeId::ERROR
    }

    fn check_reference(&mut self, mutable: bool, expr: &Expr, _span: &Span) -> TypeId {
        let inner_ty = self.check_expr(expr);
        self.interner.intern(Type::Reference {
            mutable,
            inner: inner_ty,
        })
    }

    fn check_error_propagation(&mut self, expr: &Expr, span: &Span) -> TypeId {
        let inner_ty = self.check_expr(expr);
        if inner_ty == TypeId::ERROR {
            return TypeId::ERROR;
        }

        let resolved = self.interner.resolve(self.resolve_type(inner_ty)).clone();
        match resolved {
            Type::Result(ok, _err) => ok,
            Type::Option(inner) => inner,
            _ => {
                self.errors.push(CompileError::new(
                    "E2013",
                    "`?` can only be applied to Result or Option types",
                    span.clone(),
                ));
                TypeId::ERROR
            }
        }
    }

    fn check_closure(
        &mut self,
        params: &[ClosureParam],
        ret: &Option<TypeExpr>,
        body: &Expr,
        _span: &Span,
    ) -> TypeId {
        self.symbols.push_scope(ScopeKind::Function);

        let mut param_types = Vec::new();
        for param in params {
            let ty = if let Some(ref ty_expr) = param.ty {
                self.resolve_type_expr(ty_expr)
            } else {
                self.fresh_type_var()
            };
            param_types.push(ty);
            let info = SymbolInfo {
                name: param.name.clone(),
                ty,
                kind: SymbolKind::Parameter,
                mutable: false,
                span: param.span.clone(),
                scope: self.symbols.current_scope(),
                visible: false,
            };
            let _ = self.symbols.define(param.name.clone(), info);
        }

        let ret_ty = if let Some(ref ret_expr) = ret {
            self.resolve_type_expr(ret_expr)
        } else {
            self.fresh_type_var()
        };

        let body_ty = self.check_expr(body);
        if let Err(e) = self.unify(body_ty, ret_ty, &body.span) {
            self.errors.push(e);
        }

        self.symbols.pop_scope();

        self.interner.intern(Type::Function {
            params: param_types,
            ret: ret_ty,
        })
    }

    fn check_assignment(
        &mut self,
        target: &Expr,
        op: &AssignOp,
        value: &Expr,
        span: &Span,
    ) -> TypeId {
        let target_ty = self.check_expr(target);
        let value_ty = self.check_expr(value);

        if target_ty == TypeId::ERROR || value_ty == TypeId::ERROR {
            return TypeId::UNIT;
        }

        // Check mutability for identifier targets
        if let ExprKind::Identifier(ref name) = target.kind {
            if let Some(sym_id) = self.symbols.lookup(name) {
                if !self.symbols.get_symbol(sym_id).mutable {
                    self.errors.push(CompileError::new(
                        "E2015",
                        format!("cannot assign to `{}`", name),
                        span.clone(),
                    ).with_note("variable is not declared as mutable".to_string()));
                }
            }
        }

        match op {
            AssignOp::Assign => {
                if let Err(e) = self.unify(target_ty, value_ty, span) {
                    self.errors.push(e);
                }
            }
            AssignOp::AddAssign | AssignOp::SubAssign | AssignOp::MulAssign | AssignOp::DivAssign => {
                let resolved = self.interner.resolve(self.resolve_type(target_ty)).clone();
                if !resolved.is_numeric() {
                    self.errors.push(CompileError::new(
                        "E2002",
                        format!("cannot apply compound assignment to `{}`", self.type_name(target_ty)),
                        span.clone(),
                    ));
                } else if let Err(e) = self.unify(target_ty, value_ty, span) {
                    self.errors.push(e);
                }
            }
        }

        TypeId::UNIT
    }

    // ═══════════════════════════════════════════════════════════
    // Pattern checking
    // ═══════════════════════════════════════════════════════════

    fn check_pattern(&mut self, pattern: &Pattern, expected_ty: TypeId) {
        match &pattern.kind {
            PatternKind::Identifier(name) => {
                // Update the symbol's type if it exists
                if let Some(sym_id) = self.symbols.lookup(name) {
                    let sym_ty = self.symbols.get_symbol(sym_id).ty;
                    match self.unify(sym_ty, expected_ty, &pattern.span) {
                        Ok(unified) => {
                            self.symbols.get_symbol_mut(sym_id).ty = unified;
                        }
                        Err(e) => self.errors.push(e),
                    }
                }
            }
            PatternKind::Literal(lit) => {
                let lit_ty = match lit {
                    Literal::Int(_) => TypeId::INT64,
                    Literal::Float(_) => TypeId::FLOAT64,
                    Literal::String(_) => TypeId::STRING,
                    Literal::Bool(_) => TypeId::BOOL,
                    Literal::Char(_) => TypeId::CHAR,
                };
                if let Err(e) = self.unify(lit_ty, expected_ty, &pattern.span) {
                    self.errors.push(e);
                }
            }
            PatternKind::Wildcard => {
                // Wildcard matches any type
            }
            PatternKind::Tuple(pats) => {
                let resolved = self.interner.resolve(self.resolve_type(expected_ty)).clone();
                if let Type::Tuple(ref elems) = resolved {
                    if pats.len() != elems.len() {
                        self.errors.push(CompileError::new(
                            "E2001",
                            format!(
                                "expected tuple of {} elements, found {}",
                                elems.len(),
                                pats.len()
                            ),
                            pattern.span.clone(),
                        ));
                    } else {
                        let elems = elems.clone();
                        for (pat, elem_ty) in pats.iter().zip(elems.iter()) {
                            self.check_pattern(pat, *elem_ty);
                        }
                    }
                } else if let Type::TypeVar(_) = resolved {
                    // Generate tuple type vars
                    let elem_tys: Vec<TypeId> = pats.iter().map(|_| self.fresh_type_var()).collect();
                    let tuple_ty = self.interner.intern(Type::Tuple(elem_tys.clone()));
                    if let Err(e) = self.unify(expected_ty, tuple_ty, &pattern.span) {
                        self.errors.push(e);
                    }
                    for (pat, ty) in pats.iter().zip(elem_tys.iter()) {
                        self.check_pattern(pat, *ty);
                    }
                }
            }
            PatternKind::Struct { fields, .. } => {
                // Check each field pattern against expected struct type
                let resolved = self.interner.resolve(self.resolve_type(expected_ty)).clone();
                if let Type::Struct { fields: ref def_fields, .. } = resolved {
                    let def_fields = def_fields.clone();
                    for field_pat in fields {
                        if let Some((_, fty)) = def_fields.iter().find(|(n, _)| *n == field_pat.name) {
                            if let Some(ref inner_pat) = field_pat.pattern {
                                self.check_pattern(inner_pat, *fty);
                            }
                        }
                    }
                }
            }
            PatternKind::EnumVariant { fields, .. } => {
                // Check nested patterns with fresh type vars
                for field in fields {
                    let ty = self.fresh_type_var();
                    self.check_pattern(field, ty);
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Type expression resolution
    // ═══════════════════════════════════════════════════════════

    fn resolve_type_expr(&mut self, ty: &TypeExpr) -> TypeId {
        match &ty.kind {
            TypeExprKind::Named(name) => {
                if let Some(prim) = prim_from_name(name) {
                    return self.interner.intern(Type::Primitive(prim));
                }
                if let Some(sym_id) = self.symbols.lookup(name) {
                    return self.symbols.get_symbol(sym_id).ty;
                }
                TypeId::ERROR
            }
            TypeExprKind::Path(segments) => {
                if let Some(first) = segments.first() {
                    if let Some(prim) = prim_from_name(first) {
                        if segments.len() == 1 {
                            return self.interner.intern(Type::Primitive(prim));
                        }
                    }
                }
                let path: Vec<String> = segments.clone();
                self.interner.intern(Type::Named { path, args: Vec::new() })
            }
            TypeExprKind::Generic { name, args } => {
                match name.as_str() {
                    "Option" => {
                        if let Some(TypeArg::Type(ref inner)) = args.first() {
                            let inner_id = self.resolve_type_expr(inner);
                            return self.interner.intern(Type::Option(inner_id));
                        }
                        self.fresh_type_var()
                    }
                    "Result" => {
                        let ok_id = args.first()
                            .and_then(|a| match a { TypeArg::Type(t) => Some(self.resolve_type_expr(t)), _ => None })
                            .unwrap_or_else(|| self.fresh_type_var());
                        let err_id = args.get(1)
                            .and_then(|a| match a { TypeArg::Type(t) => Some(self.resolve_type_expr(t)), _ => None })
                            .unwrap_or_else(|| self.fresh_type_var());
                        self.interner.intern(Type::Result(ok_id, err_id))
                    }
                    _ => {
                        let resolved_args: Vec<TypeId> = args.iter()
                            .filter_map(|a| match a { TypeArg::Type(t) => Some(self.resolve_type_expr(t)), _ => None })
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
                let resolved_shape: Vec<ShapeDimResolved> = shape.iter().map(|dim| match dim {
                    ShapeDim::Constant(n) => ShapeDimResolved::Known(*n),
                    ShapeDim::Dynamic => ShapeDimResolved::Dynamic,
                    ShapeDim::Named(name) => ShapeDimResolved::Variable(name.clone()),
                }).collect();
                self.interner.intern(Type::Tensor(TensorType { dtype: dtype_id, shape: resolved_shape }))
            }
            TypeExprKind::Reference { mutable, inner } => {
                let inner_id = self.resolve_type_expr(inner);
                self.interner.intern(Type::Reference { mutable: *mutable, inner: inner_id })
            }
            TypeExprKind::Function { params, return_type } => {
                let param_ids: Vec<TypeId> = params.iter().map(|p| self.resolve_type_expr(p)).collect();
                let ret_id = self.resolve_type_expr(return_type);
                self.interner.intern(Type::Function { params: param_ids, ret: ret_id })
            }
            TypeExprKind::Tuple(types) => {
                let ids: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                self.interner.intern(Type::Tuple(ids))
            }
            TypeExprKind::Array { element, size } => {
                let elem_id = self.resolve_type_expr(element);
                self.interner.intern(Type::Array { elem: elem_id, size: *size })
            }
            TypeExprKind::Inferred => self.fresh_type_var(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Public entry point
// ═══════════════════════════════════════════════════════════════

/// Parse, resolve names, and type-check an Axon source string.
pub fn check(source: &str, filename: &str) -> (TypeChecker, Vec<CompileError>) {
    let (program, parse_errors) = crate::parse_source(source, filename);
    let mut checker = TypeChecker::new();
    checker.errors.extend(parse_errors);

    // Register standard library types and functions before user code
    crate::stdlib::register_stdlib(&mut checker.interner, &mut checker.symbols);

    // Run name resolution first
    {
        let mut resolver = NameResolver::new(&mut checker.symbols, &mut checker.interner);
        let resolve_errors = resolver.resolve(&program);
        checker.errors.extend(resolve_errors);
    }

    // Then type check (only if no hard errors so far)
    if !checker.has_errors() {
        checker.check_program(&program);
    }

    let errors = checker.errors.clone();
    (checker, errors)
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn check_ok(source: &str) -> TypeChecker {
        let (checker, errors) = check(source, "test.axon");
        assert!(
            errors.is_empty(),
            "expected no errors, got: {:?}",
            errors.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
        checker
    }

    fn check_err(source: &str) -> Vec<CompileError> {
        let (_checker, errors) = check(source, "test.axon");
        assert!(!errors.is_empty(), "expected errors but got none");
        errors
    }

    fn check_has_error_code(source: &str, code: &str) -> Vec<CompileError> {
        let (_checker, errors) = check(source, "test.axon");
        assert!(
            errors.iter().any(|e| e.error_code == code),
            "expected error code {}, got: {:?}",
            code,
            errors.iter().map(|e| format!("{}: {}", e.error_code, e.message)).collect::<Vec<_>>()
        );
        errors
    }

    // ── 1. Infer integer literal type ──────────────────────────

    #[test]
    fn infer_integer_literal() {
        let src = "fn main() -> Int64 { return 42; }";
        check_ok(src);
    }

    // ── 2. Infer float literal type ────────────────────────────

    #[test]
    fn infer_float_literal() {
        let src = "fn main() -> Float64 { return 3.14; }";
        check_ok(src);
    }

    // ── 3. Binary op on same types ─────────────────────────────

    #[test]
    fn binary_op_same_types() {
        let src = "fn add(a: Int64, b: Int64) -> Int64 { return a + b; }";
        check_ok(src);
    }

    // ── 4. Binary op type mismatch error ───────────────────────

    #[test]
    fn binary_op_type_mismatch() {
        let src = "fn bad(a: Int64, b: Bool) { return a + b; }";
        check_has_error_code(src, "E2001");
    }

    // ── 5. Function call type checking ─────────────────────────

    #[test]
    fn function_call_type_check() {
        let src = "fn double(x: Int64) -> Int64 { return x + x; }\nfn main() -> Int64 { return double(5); }";
        check_ok(src);
    }

    // ── 6. Let with type annotation match ──────────────────────

    #[test]
    fn let_with_annotation_match() {
        let src = r#"
            fn main() {
                let x: Int64 = 42;
            }
        "#;
        check_ok(src);
    }

    // ── 7. Let with type annotation mismatch error ─────────────

    #[test]
    fn let_with_annotation_mismatch() {
        let src = r#"
            fn main() {
                let x: Bool = 42;
            }
        "#;
        check_has_error_code(src, "E2001");
    }

    // ── 8. If-else type unification ────────────────────────────

    #[test]
    fn if_else_type_unification() {
        // Both branches return the same type via return statements
        let src = "fn pick(cond: Bool) -> Int64 { if cond { return 1; } else { return 2; } }";
        check_ok(src);
    }

    // ── 9. If-else branch mismatch via return ──────────────────

    #[test]
    fn if_else_branch_mismatch() {
        // Return type mismatch: Int64 vs Bool against declared return Int64
        let src = "fn pick(cond: Bool) -> Int64 { if cond { return 1; } else { return true; } }";
        check_has_error_code(src, "E2001");
    }

    // ── 10. Unification of type variables ──────────────────────

    #[test]
    fn type_var_unification() {
        let src = "fn identity(x: Int64) -> Int64 { let y = x; return y; }";
        check_ok(src);
    }

    // ── 11. Occurs check (infinite type) ───────────────────────

    #[test]
    fn occurs_check_basic() {
        // Test the occurs check directly on the TypeChecker
        let mut tc = TypeChecker::new();
        let var_id = 999u32;
        let var_ty = tc.interner.intern(Type::TypeVar(var_id));
        // A type that contains the variable: Tuple([TypeVar(999)])
        let tuple_ty = tc.interner.intern(Type::Tuple(vec![var_ty]));
        assert!(tc.occurs_check(var_id, tuple_ty));
        // A type that does NOT contain the variable
        assert!(!tc.occurs_check(var_id, TypeId::INT32));
    }

    // ── 12. Pattern matching type check ────────────────────────

    #[test]
    fn match_type_check() {
        let src = "fn test(x: Int64) -> Int64 { return match x { 1 => 10, 2 => 20, _ => 0, }; }";
        check_ok(src);
    }

    // ── 13. Struct literal type checking ───────────────────────

    #[test]
    fn struct_literal_type_check() {
        let src = "struct Point { x: Int64, y: Int64 }\nfn make() { let p = Point { x: 1, y: 2 }; }";
        check_ok(src);
    }

    // ── 14. Reference type checking ────────────────────────────

    #[test]
    fn reference_type_check() {
        let src = r#"
            fn test(x: Int64) {
                let r = &x;
            }
        "#;
        check_ok(src);
    }

    // ── 15. Type coercion: &mut to & is ok ─────────────────────

    #[test]
    fn ref_coercion_mut_to_immut() {
        // Test directly: unifying &mut T with &T should succeed
        let mut tc = TypeChecker::new();
        let mut_ref = tc.interner.intern(Type::Reference { mutable: true, inner: TypeId::INT32 });
        let imm_ref = tc.interner.intern(Type::Reference { mutable: false, inner: TypeId::INT32 });
        let span = Span::dummy();
        let result = tc.unify(mut_ref, imm_ref, &span);
        assert!(result.is_ok());
        // Result should be &Int32 (immutable)
        let unified = result.unwrap();
        let ty = tc.interner.resolve(unified);
        if let Type::Reference { mutable, .. } = ty {
            assert!(!mutable, "unified reference should be immutable");
        } else {
            panic!("expected Reference type");
        }
    }

    // ── 16. Comparison returns Bool ────────────────────────────

    #[test]
    fn comparison_returns_bool() {
        let src = "fn test(a: Int64, b: Int64) -> Bool { return a < b; }";
        check_ok(src);
    }

    // ── 17. Logical ops require Bool ───────────────────────────

    #[test]
    fn logical_ops_require_bool() {
        let src = "fn test(a: Bool, b: Bool) -> Bool { return a && b; }";
        check_ok(src);
    }

    // ── 18. Cannot negate Bool ──────────────────────────────────

    #[test]
    fn cannot_negate_non_numeric() {
        let src = "fn test(a: Bool) { return -a; }";
        check_has_error_code(src, "E2002");
    }

    // ── 19. Wrong number of arguments ──────────────────────────

    #[test]
    fn wrong_arg_count() {
        let src = "fn foo(x: Int64) -> Int64 { return x; }\nfn main() { foo(1, 2); }";
        check_has_error_code(src, "E2003");
    }

    // ── 20. Match arm mismatch ─────────────────────────────────

    #[test]
    fn match_arm_mismatch() {
        let src = "fn test(x: Int64) { return match x { 1 => 10, _ => true, }; }";
        check_has_error_code(src, "E2009");
    }

    // ── 21. Assignment to immutable ────────────────────────────

    #[test]
    fn assignment_to_immutable() {
        let src = r#"
            fn test() {
                let x: Int64 = 1;
                x = 2;
            }
        "#;
        check_has_error_code(src, "E2015");
    }

    // ── 22. Mutable assignment ok ──────────────────────────────

    #[test]
    fn mutable_assignment_ok() {
        let src = r#"
            fn test() {
                let mut x: Int64 = 1;
                x = 2;
            }
        "#;
        check_ok(src);
    }

    // ── 23. Boolean not ────────────────────────────────────────

    #[test]
    fn boolean_not() {
        let src = "fn test(a: Bool) -> Bool { return !a; }";
        check_ok(src);
    }

    // ── 24. While condition must be Bool ────────────────────────

    #[test]
    fn while_condition_must_be_bool() {
        let src = "fn test() { while 42 { } }";
        check_has_error_code(src, "E2001");
    }

    // ── 25. Tuple type checking ────────────────────────────────

    #[test]
    fn tuple_type_check() {
        let src = "fn test() -> (Int64, Bool) { return (42, true); }";
        check_ok(src);
    }
}
