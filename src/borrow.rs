//! # Borrow Checker
//!
//! Current implementation uses AST-walk based analysis rather than CFG-based
//! dataflow analysis. This correctly catches common borrow violations (use-after-move,
//! double-mutable-borrow, borrow-while-mutably-borrowed) but may miss complex
//! control-flow dependent violations.
//!
//! **Planned improvement:** Migrate to MIR-based dataflow analysis (similar to
//! rustc's borrow checker) which operates on the control flow graph for
//! path-sensitive precision. Tracked for post-v1.0 improvement.

// borrow.rs — Borrow checking and move analysis (Phase 3e)

use std::collections::HashMap;

use crate::ast::*;
use crate::error::CompileError;
use crate::span::Span;
use crate::symbol::*;
use crate::types::*;

// ═══════════════════════════════════════════════════════════════
// Data structures
// ═══════════════════════════════════════════════════════════════

/// Unique identifier for a "place" (variable or path like x.field).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    pub base: String,
    pub projections: Vec<String>,
}

impl Place {
    pub fn var(name: &str) -> Self {
        Place {
            base: name.to_string(),
            projections: Vec::new(),
        }
    }

    pub fn with_field(mut self, field: &str) -> Self {
        self.projections.push(field.to_string());
        self
    }

    /// True if `self` is a prefix of (or equal to) `other`.
    pub fn overlaps(&self, other: &Place) -> bool {
        if self.base != other.base {
            return false;
        }
        let min_len = self.projections.len().min(other.projections.len());
        self.projections[..min_len] == other.projections[..min_len]
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base)?;
        for proj in &self.projections {
            write!(f, ".{}", proj)?;
        }
        Ok(())
    }
}

/// A borrow record.
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub place: Place,
    pub mutable: bool,
    pub span: Span,
    pub alive: bool,
    pub scope_depth: usize,
}

/// Move/copy tracking for a variable.
#[derive(Debug, Clone)]
pub struct MoveInfo {
    pub name: String,
    pub moved_at: Option<Span>,
    pub is_copy: bool,
    pub ty: TypeId,
}

/// Device annotation for tensors.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceContext {
    Cpu,
    Gpu,
    Device,
    Unknown,
}

/// Basic block in the CFG.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub stmts: Vec<BorrowStmt>,
    pub successors: Vec<usize>,
}

/// Simplified statement for borrow analysis.
#[derive(Debug, Clone)]
pub enum BorrowStmt {
    Assign { place: Place, rhs: BorrowRhs, span: Span },
    Use { place: Place, span: Span },
    Borrow { place: Place, mutable: bool, span: Span },
    Move { place: Place, span: Span },
    Drop { place: Place, span: Span },
    Return { span: Span },
}

#[derive(Debug, Clone)]
pub enum BorrowRhs {
    Use(Place),
    Borrow(Place, bool),
    Move(Place),
    Copy(Place),
    Literal,
    Call,
}

// ═══════════════════════════════════════════════════════════════
// BorrowChecker
// ═══════════════════════════════════════════════════════════════

pub struct BorrowChecker<'a> {
    interner: &'a TypeInterner,
    symbols: &'a SymbolTable,
    errors: Vec<CompileError>,

    borrows: Vec<BorrowInfo>,
    moves: HashMap<String, MoveInfo>,
    current_device: DeviceContext,
    scope_depth: usize,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(interner: &'a TypeInterner, symbols: &'a SymbolTable) -> Self {
        BorrowChecker {
            interner,
            symbols,
            errors: Vec::new(),
            borrows: Vec::new(),
            moves: HashMap::new(),
            current_device: DeviceContext::Unknown,
            scope_depth: 0,
        }
    }

    pub fn take_errors(self) -> Vec<CompileError> {
        self.errors
    }

    // ── Program / item traversal ──────────────────────────────

    pub fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function(decl) => {
                self.check_function(decl, &item.attributes);
            }
            ItemKind::Impl(impl_block) => {
                for sub_item in &impl_block.items {
                    self.check_item(sub_item);
                }
            }
            ItemKind::Trait(trait_decl) => {
                for sub_item in &trait_decl.items {
                    self.check_item(sub_item);
                }
            }
            _ => {}
        }
    }

    fn check_function(&mut self, decl: &FnDecl, attrs: &[Attribute]) {
        let saved_borrows = std::mem::take(&mut self.borrows);
        let saved_moves = std::mem::take(&mut self.moves);
        let saved_device = self.current_device.clone();
        let saved_depth = self.scope_depth;

        self.current_device = self.get_device_context(attrs);
        self.scope_depth = 0;
        self.enter_scope();

        // Register parameters as live variables
        for param in &decl.params {
            match &param.kind {
                FnParamKind::Typed { name, .. } => {
                    let ty = self.resolve_param_type(name);
                    let is_copy = self.is_copy_type(ty);
                    self.moves.insert(
                        name.clone(),
                        MoveInfo {
                            name: name.clone(),
                            moved_at: None,
                            is_copy,
                            ty,
                        },
                    );
                }
                FnParamKind::SelfOwned | FnParamKind::SelfRef | FnParamKind::SelfMutRef => {
                    self.moves.insert(
                        "self".to_string(),
                        MoveInfo {
                            name: "self".to_string(),
                            moved_at: None,
                            is_copy: false,
                            ty: TypeId::UNIT, // placeholder
                        },
                    );
                }
            }
        }

        if let Some(body) = &decl.body {
            self.check_block(body);
        }

        self.exit_scope();
        self.borrows = saved_borrows;
        self.moves = saved_moves;
        self.current_device = saved_device;
        self.scope_depth = saved_depth;
    }

    // ── Block / statement / expression ────────────────────────

    fn check_block(&mut self, block: &Block) {
        self.enter_scope();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(tail) = &block.tail_expr {
            self.check_expr(tail);
        }
        self.exit_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { name, mutable, initializer, .. } => {
                if let Some(init) = initializer {
                    self.check_expr(init);
                }
                self.bind_pattern(name, self.infer_pattern_type(name), *mutable);
            }
            StmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            StmtKind::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.check_expr(expr);
                }
            }
            StmtKind::While { condition, body } => {
                self.check_expr(condition);
                self.check_block(body);
            }
            StmtKind::For { pattern, iterator, body } => {
                self.check_expr(iterator);
                self.enter_scope();
                self.bind_pattern(pattern, TypeId::UNIT, false);
                self.check_block(body);
                self.exit_scope();
            }
            StmtKind::Item(item) => {
                self.check_item(item);
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                self.check_use(name, &expr.span);
            }
            ExprKind::Path(segments) => {
                if let Some(first) = segments.first() {
                    self.check_use(first, &expr.span);
                }
            }
            ExprKind::Literal(_) => {}

            ExprKind::BinaryOp { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }
            ExprKind::UnaryOp { op, operand } => {
                match op {
                    UnaryOp::Ref => {
                        if let Some(place) = self.expr_to_place(operand) {
                            self.check_borrow_conflicts(&place, false, &expr.span);
                            self.record_borrow(place, false, &expr.span);
                        }
                        self.check_expr(operand);
                    }
                    UnaryOp::MutRef => {
                        if let Some(place) = self.expr_to_place(operand) {
                            self.check_mutability(&place.base, &expr.span);
                            self.check_borrow_conflicts(&place, true, &expr.span);
                            self.record_borrow(place, true, &expr.span);
                        }
                        self.check_expr(operand);
                    }
                    _ => {
                        self.check_expr(operand);
                    }
                }
            }

            ExprKind::Reference { mutable, expr: inner } => {
                if let Some(place) = self.expr_to_place(inner) {
                    if *mutable {
                        self.check_mutability(&place.base, &expr.span);
                    }
                    self.check_borrow_conflicts(&place, *mutable, &expr.span);
                    self.record_borrow(place, *mutable, &expr.span);
                }
                self.check_expr(inner);
            }

            ExprKind::Assignment { target, op: _, value } => {
                self.check_assignment(target, value, &expr.span);
            }

            ExprKind::FnCall { function, args } => {
                self.check_expr(function);
                for arg in args {
                    self.handle_value_use(arg);
                }
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                self.check_expr(receiver);
                for arg in args {
                    self.handle_value_use(arg);
                }
            }

            ExprKind::FieldAccess { object, .. } => {
                self.check_expr(object);
            }
            ExprKind::Index { object, index } => {
                self.check_expr(object);
                self.check_expr(index);
            }
            ExprKind::Slice { object, start, end } => {
                self.check_expr(object);
                if let Some(s) = start {
                    self.check_expr(s);
                }
                if let Some(e) = end {
                    self.check_expr(e);
                }
            }

            ExprKind::IfElse { condition, then_block, else_block } => {
                self.check_expr(condition);
                self.check_block(then_block);
                if let Some(else_clause) = else_block {
                    match else_clause {
                        ElseClause::ElseBlock(block) => self.check_block(block),
                        ElseClause::ElseIf(else_if) => self.check_expr(else_if),
                    }
                }
            }

            ExprKind::Match { expr: scrutinee, arms } => {
                self.check_expr(scrutinee);
                for arm in arms {
                    self.enter_scope();
                    self.bind_pattern(&arm.pattern, TypeId::UNIT, false);
                    if let Some(guard) = &arm.guard {
                        self.check_expr(guard);
                    }
                    self.check_expr(&arm.body);
                    self.exit_scope();
                }
            }

            ExprKind::Block(block) => {
                self.check_block(block);
            }

            ExprKind::Tuple(elems) => {
                for elem in elems {
                    self.check_expr(elem);
                }
            }

            ExprKind::StructLiteral { fields, .. } => {
                for field in fields {
                    self.handle_value_use(&field.value);
                }
            }

            ExprKind::Closure { body, .. } => {
                self.check_expr(body);
            }

            ExprKind::TypeCast { expr: inner, .. } => {
                self.check_expr(inner);
            }
            ExprKind::ErrorPropagation(inner) => {
                self.check_expr(inner);
            }
            ExprKind::Range { start, end } => {
                if let Some(s) = start {
                    self.check_expr(s);
                }
                if let Some(e) = end {
                    self.check_expr(e);
                }
            }
        }
    }

    // ── Core borrow checking logic ────────────────────────────

    fn record_move(&mut self, name: &str, ty: TypeId, span: &Span) {
        if let Some(info) = self.moves.get_mut(name) {
            if !info.is_copy && info.moved_at.is_none() {
                info.moved_at = Some(span.clone());
            }
        } else {
            let is_copy = self.is_copy_type(ty);
            self.moves.insert(
                name.to_string(),
                MoveInfo {
                    name: name.to_string(),
                    moved_at: if is_copy { None } else { Some(span.clone()) },
                    is_copy,
                    ty,
                },
            );
        }
    }

    /// E4001: use of moved value.
    fn check_use(&mut self, name: &str, span: &Span) {
        if let Some(info) = self.moves.get(name) {
            if let Some(ref moved_span) = info.moved_at {
                self.errors.push(
                    CompileError::new(
                        "E4001",
                        format!("use of moved value `{}`", name),
                        span.clone(),
                    )
                    .with_note(format!(
                        "value moved here: {}:{}:{}",
                        moved_span.start.file, moved_span.start.line, moved_span.start.column,
                    ))
                    .with_suggestion("consider cloning the value before moving it"),
                );
            }
        }
    }

    fn record_borrow(&mut self, place: Place, mutable: bool, span: &Span) {
        self.borrows.push(BorrowInfo {
            place,
            mutable,
            span: span.clone(),
            alive: true,
            scope_depth: self.scope_depth,
        });
    }

    /// E4002: cannot borrow as mutable while immutable borrow exists.
    /// E4003: cannot borrow as mutable more than once.
    fn check_borrow_conflicts(&mut self, place: &Place, mutable: bool, span: &Span) {
        let name = &place.base;
        for borrow in &self.borrows {
            if !borrow.alive || !borrow.place.overlaps(place) {
                continue;
            }
            if mutable && !borrow.mutable {
                // E4002: trying to borrow mutably while immutable borrow exists
                self.errors.push(
                    CompileError::new(
                        "E4002",
                        format!(
                            "cannot borrow `{}` as mutable because it is also borrowed as immutable",
                            name
                        ),
                        span.clone(),
                    )
                    .with_note(format!(
                        "immutable borrow created here: {}:{}:{}",
                        borrow.span.start.file,
                        borrow.span.start.line,
                        borrow.span.start.column,
                    )),
                );
                return;
            }
            if mutable && borrow.mutable {
                // E4003: double mutable borrow
                self.errors.push(
                    CompileError::new(
                        "E4003",
                        format!(
                            "cannot borrow `{}` as mutable more than once at a time",
                            name
                        ),
                        span.clone(),
                    )
                    .with_note(format!(
                        "first mutable borrow here: {}:{}:{}",
                        borrow.span.start.file,
                        borrow.span.start.line,
                        borrow.span.start.column,
                    )),
                );
                return;
            }
            if !mutable && borrow.mutable {
                // E4002 symmetric: trying to borrow immutably while mutable borrow exists
                self.errors.push(
                    CompileError::new(
                        "E4002",
                        format!(
                            "cannot borrow `{}` as immutable because it is also borrowed as mutable",
                            name
                        ),
                        span.clone(),
                    )
                    .with_note(format!(
                        "mutable borrow created here: {}:{}:{}",
                        borrow.span.start.file,
                        borrow.span.start.line,
                        borrow.span.start.column,
                    )),
                );
                return;
            }
        }
    }

    /// E4004: assignment to immutable variable.
    fn check_mutability(&mut self, name: &str, span: &Span) {
        if let Some(sym_id) = self.symbols.lookup(name) {
            let info = self.symbols.get_symbol(sym_id);
            if !info.mutable
                && matches!(info.kind, SymbolKind::Variable | SymbolKind::Parameter)
            {
                self.errors.push(
                    CompileError::new(
                        "E4004",
                        format!("cannot assign to immutable variable `{}`", name),
                        span.clone(),
                    )
                    .with_suggestion(format!(
                        "consider making this variable mutable: `let mut {}`",
                        name
                    )),
                );
            }
        }
    }

    fn release_borrows_in_scope(&mut self) {
        let depth = self.scope_depth;
        for borrow in &mut self.borrows {
            if borrow.alive && borrow.scope_depth >= depth {
                borrow.alive = false;
            }
        }
    }

    // ── Move semantics ────────────────────────────────────────

    pub fn is_copy_type(&self, ty: TypeId) -> bool {
        match self.interner.resolve(ty) {
            Type::Primitive(p) => p.is_copy(),
            Type::Unit | Type::Never => true,
            Type::Tuple(elems) => elems.iter().all(|e| self.is_copy_type(*e)),
            Type::Error => true, // don't cascade errors
            _ => false,
        }
    }

    fn handle_value_use(&mut self, expr: &Expr) {
        self.check_expr(expr);
        if let ExprKind::Identifier(name) = &expr.kind {
            let ty = self.resolve_var_type(name);
            if !self.is_copy_type(ty) {
                self.record_move(name, ty, &expr.span);
            }
        }
    }

    // ── Device-aware checks ───────────────────────────────────

    fn get_device_context(&self, attrs: &[Attribute]) -> DeviceContext {
        for attr in attrs {
            match attr {
                Attribute::Cpu => return DeviceContext::Cpu,
                Attribute::Gpu => return DeviceContext::Gpu,
                Attribute::Device(_) => return DeviceContext::Device,
            }
        }
        DeviceContext::Unknown
    }

    /// E4006: device transfer without explicit copy.
    fn check_device_transfer(
        &mut self,
        name: &str,
        from: &DeviceContext,
        to: &DeviceContext,
        span: &Span,
    ) {
        if from != to
            && *from != DeviceContext::Unknown
            && *to != DeviceContext::Unknown
            && *from != DeviceContext::Device
            && *to != DeviceContext::Device
        {
            let (from_str, to_str) = match (from, to) {
                (DeviceContext::Gpu, DeviceContext::Cpu) => ("@gpu", "@cpu"),
                (DeviceContext::Cpu, DeviceContext::Gpu) => ("@cpu", "@gpu"),
                _ => return,
            };
            self.errors.push(
                CompileError::new(
                    "E4006",
                    format!(
                        "cannot transfer {} tensor `{}` to {} context without explicit copy",
                        from_str, name, to_str
                    ),
                    span.clone(),
                )
                .with_suggestion(format!(
                    "use `.to_{}()` for explicit copy",
                    if *to == DeviceContext::Cpu { "cpu" } else { "gpu" }
                )),
            );
        }
    }

    /// E4007: cannot borrow @gpu tensor as &mut from @cpu.
    fn check_device_borrow(
        &mut self,
        name: &str,
        mutable: bool,
        var_device: &DeviceContext,
        span: &Span,
    ) {
        if mutable && *var_device == DeviceContext::Gpu && self.current_device == DeviceContext::Cpu
        {
            self.errors.push(CompileError::new(
                "E4007",
                format!(
                    "cannot borrow @gpu tensor `{}` as `&mut` from @cpu function",
                    name
                ),
                span.clone(),
            ));
        }
    }

    // ── Lifetime (simplified) ─────────────────────────────────

    /// E4005: reference escapes scope.
    fn check_reference_escape(
        &mut self,
        name: &str,
        ref_scope: usize,
        current_scope: usize,
        span: &Span,
    ) {
        if ref_scope > current_scope {
            self.errors.push(CompileError::new(
                "E4005",
                format!("reference to `{}` escapes its scope", name),
                span.clone(),
            ));
        }
    }

    // ── Assignment ────────────────────────────────────────────

    fn check_assignment(&mut self, target: &Expr, value: &Expr, span: &Span) {
        // Check value first
        self.check_expr(value);

        // Check the target is mutable
        if let Some(name) = self.expr_to_name(target) {
            self.check_mutability(&name, span);
            // Also check borrow conflicts on the target
            let place = Place::var(&name);
            self.check_borrow_conflicts(&place, true, span);
        }

        // If the value is a variable, it may move
        if let ExprKind::Identifier(name) = &value.kind {
            let ty = self.resolve_var_type(name);
            if !self.is_copy_type(ty) {
                self.record_move(name, ty, &value.span);
            }
        }
    }

    // ── Pattern binding ───────────────────────────────────────

    fn bind_pattern(&mut self, pattern: &Pattern, ty: TypeId, mutable: bool) {
        match &pattern.kind {
            PatternKind::Identifier(name) => {
                let is_copy = self.is_copy_type(ty);
                self.moves.insert(
                    name.clone(),
                    MoveInfo {
                        name: name.clone(),
                        moved_at: None,
                        is_copy,
                        ty,
                    },
                );
            }
            PatternKind::Tuple(pats) => {
                for pat in pats {
                    self.bind_pattern(pat, ty, mutable);
                }
            }
            PatternKind::Struct { fields, .. } => {
                for field in fields {
                    if let Some(ref pat) = field.pattern {
                        self.bind_pattern(pat, ty, mutable);
                    } else {
                        // Shorthand: `Point { x, y }` binds x and y
                        self.moves.insert(
                            field.name.clone(),
                            MoveInfo {
                                name: field.name.clone(),
                                moved_at: None,
                                is_copy: self.is_copy_type(ty),
                                ty,
                            },
                        );
                    }
                }
            }
            PatternKind::EnumVariant { fields, .. } => {
                for pat in fields {
                    self.bind_pattern(pat, ty, mutable);
                }
            }
            PatternKind::Wildcard | PatternKind::Literal(_) => {}
        }
    }

    // ── Scope management ──────────────────────────────────────

    fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn exit_scope(&mut self) {
        self.release_borrows_in_scope();
        self.scope_depth = self.scope_depth.saturating_sub(1);
    }

    // ── Helpers ───────────────────────────────────────────────

    fn expr_to_place(&self, expr: &Expr) -> Option<Place> {
        match &expr.kind {
            ExprKind::Identifier(name) => Some(Place::var(name)),
            ExprKind::FieldAccess { object, field } => {
                self.expr_to_place(object).map(|p| p.with_field(field))
            }
            _ => None,
        }
    }

    fn expr_to_name(&self, expr: &Expr) -> Option<String> {
        match &expr.kind {
            ExprKind::Identifier(name) => Some(name.clone()),
            ExprKind::FieldAccess { object, .. } => self.expr_to_name(object),
            _ => None,
        }
    }

    fn resolve_var_type(&self, name: &str) -> TypeId {
        if let Some(sym_id) = self.symbols.lookup(name) {
            self.symbols.get_symbol(sym_id).ty
        } else if let Some(info) = self.moves.get(name) {
            info.ty
        } else {
            TypeId::ERROR
        }
    }

    fn resolve_param_type(&self, name: &str) -> TypeId {
        if let Some(sym_id) = self.symbols.lookup(name) {
            self.symbols.get_symbol(sym_id).ty
        } else {
            TypeId::ERROR
        }
    }

    fn infer_pattern_type(&self, pattern: &Pattern) -> TypeId {
        match &pattern.kind {
            PatternKind::Identifier(name) => self.resolve_var_type(name),
            _ => TypeId::UNIT,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Public entry point
// ═══════════════════════════════════════════════════════════════

/// Run borrow checking on a fully type-checked program.
pub fn borrow_check(
    program: &Program,
    interner: &TypeInterner,
    symbols: &SymbolTable,
) -> Vec<CompileError> {
    let mut checker = BorrowChecker::new(interner, symbols);
    checker.check_program(program);
    checker.take_errors()
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TypeInterner, Type, PrimKind};

    // ── Test helpers ──────────────────────────────────────────

    fn make_checker<'a>(
        interner: &'a TypeInterner,
        symbols: &'a SymbolTable,
    ) -> BorrowChecker<'a> {
        BorrowChecker::new(interner, symbols)
    }

    fn dummy_span() -> Span {
        Span::dummy()
    }

    fn span_at(line: usize, col: usize) -> Span {
        Span::new("<test>", line, col, line, col + 1)
    }

    // ── E4001: use of moved value ─────────────────────────────

    #[test]
    fn use_after_move_detected() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        // Register a non-copy variable
        bc.moves.insert(
            "x".to_string(),
            MoveInfo {
                name: "x".to_string(),
                moved_at: None,
                is_copy: false,
                ty: TypeId::STRING,
            },
        );

        // Move it
        bc.record_move("x", TypeId::STRING, &span_at(1, 5));
        // Try to use it after move
        bc.check_use("x", &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4001");
        assert!(errors[0].message.contains("use of moved value `x`"));
    }

    #[test]
    fn copy_type_allows_reuse() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        bc.moves.insert(
            "n".to_string(),
            MoveInfo {
                name: "n".to_string(),
                moved_at: None,
                is_copy: true,
                ty: TypeId::INT32,
            },
        );

        // "Move" a copy type — should not actually mark it as moved
        bc.record_move("n", TypeId::INT32, &span_at(1, 5));
        bc.check_use("n", &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── E4002: mutable borrow conflict with immutable borrow ──

    #[test]
    fn mutable_borrow_conflict_with_immutable() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("x");
        // Immutable borrow
        bc.record_borrow(place.clone(), false, &span_at(1, 5));
        // Try mutable borrow
        bc.check_borrow_conflicts(&place, true, &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4002");
        assert!(errors[0].message.contains("cannot borrow `x` as mutable"));
    }

    // ── E4003: double mutable borrow ──────────────────────────

    #[test]
    fn double_mutable_borrow() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("y");
        bc.record_borrow(place.clone(), true, &span_at(1, 5));
        bc.check_borrow_conflicts(&place, true, &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4003");
        assert!(errors[0]
            .message
            .contains("cannot borrow `y` as mutable more than once"));
    }

    // ── E4004: assignment to immutable variable ───────────────

    #[test]
    fn assignment_to_immutable() {
        let mut interner = TypeInterner::new();
        let mut symbols = SymbolTable::new();

        // Define immutable variable "z"
        let scope = symbols.current_scope();
        let ty = TypeId::INT32;
        let _ = symbols.define(
            "z".to_string(),
            SymbolInfo {
                name: "z".to_string(),
                ty,
                kind: SymbolKind::Variable,
                mutable: false,
                span: dummy_span(),
                scope,
                visible: true,
            },
        );

        let mut bc = make_checker(&interner, &symbols);
        bc.check_mutability("z", &span_at(3, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4004");
        assert!(errors[0]
            .message
            .contains("cannot assign to immutable variable `z`"));
    }

    #[test]
    fn mutable_variable_can_be_assigned() {
        let mut interner = TypeInterner::new();
        let mut symbols = SymbolTable::new();

        let scope = symbols.current_scope();
        let _ = symbols.define(
            "w".to_string(),
            SymbolInfo {
                name: "w".to_string(),
                ty: TypeId::INT32,
                kind: SymbolKind::Variable,
                mutable: true,
                span: dummy_span(),
                scope,
                visible: true,
            },
        );

        let mut bc = make_checker(&interner, &symbols);
        bc.check_mutability("w", &span_at(3, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── Single mutable borrow is ok ───────────────────────────

    #[test]
    fn single_mutable_borrow_ok() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("a");
        bc.check_borrow_conflicts(&place, true, &span_at(1, 5));
        bc.record_borrow(place, true, &span_at(1, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── Multiple immutable borrows are ok ─────────────────────

    #[test]
    fn multiple_immutable_borrows_ok() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("b");
        bc.record_borrow(place.clone(), false, &span_at(1, 5));
        bc.check_borrow_conflicts(&place, false, &span_at(2, 5));
        bc.record_borrow(place.clone(), false, &span_at(2, 5));
        bc.check_borrow_conflicts(&place, false, &span_at(3, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── Borrow released after scope ends ──────────────────────

    #[test]
    fn borrow_released_after_scope() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope(); // depth 1
        {
            bc.enter_scope(); // depth 2
            let place = Place::var("c");
            bc.record_borrow(place.clone(), true, &span_at(1, 5));
            bc.exit_scope(); // depth 1 — borrows at depth 2 released
        }
        // Now a new mutable borrow should be fine
        let place = Place::var("c");
        bc.check_borrow_conflicts(&place, true, &span_at(3, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── E4007: @gpu tensor cannot be &mut from @cpu ───────────

    #[test]
    fn gpu_tensor_no_mut_borrow_from_cpu() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.current_device = DeviceContext::Cpu;
        bc.check_device_borrow("tensor_a", true, &DeviceContext::Gpu, &span_at(5, 10));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4007");
        assert!(errors[0]
            .message
            .contains("cannot borrow @gpu tensor `tensor_a` as `&mut` from @cpu function"));
    }

    // ── Simple function with no borrow issues ─────────────────

    #[test]
    fn simple_function_no_issues() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();

        // Empty program
        let program = Program {
            items: vec![],
            span: dummy_span(),
        };

        let errors = borrow_check(&program, &interner, &symbols);
        assert_eq!(errors.len(), 0);
    }

    // ── Move in if-else branch ────────────────────────────────

    #[test]
    fn move_in_branch_then_use() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        bc.moves.insert(
            "s".to_string(),
            MoveInfo {
                name: "s".to_string(),
                moved_at: None,
                is_copy: false,
                ty: TypeId::STRING,
            },
        );

        // Simulate move inside a branch
        bc.record_move("s", TypeId::STRING, &span_at(2, 5));
        // Subsequent use should trigger E4001
        bc.check_use("s", &span_at(4, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4001");
    }

    // ── Pattern binding moves value ───────────────────────────

    #[test]
    fn pattern_binding_moves_value() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        // Register a non-copy variable "data"
        bc.moves.insert(
            "data".to_string(),
            MoveInfo {
                name: "data".to_string(),
                moved_at: None,
                is_copy: false,
                ty: TypeId::STRING,
            },
        );

        // Simulate "let val = data;" — move data
        bc.record_move("data", TypeId::STRING, &span_at(1, 10));

        // Bind pattern "val" as non-copy
        let pat = Pattern {
            kind: PatternKind::Identifier("val".to_string()),
            span: dummy_span(),
        };
        bc.bind_pattern(&pat, TypeId::STRING, false);

        // "val" should now exist and not be moved
        assert!(bc.moves.contains_key("val"));
        assert!(bc.moves["val"].moved_at.is_none());

        // "data" should be moved
        assert!(bc.moves["data"].moved_at.is_some());

        // Using data after move triggers E4001
        bc.check_use("data", &span_at(3, 5));
        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4001");
    }

    // ── Copy types in function args don't move ────────────────

    #[test]
    fn copy_types_in_fn_args_dont_move() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        bc.moves.insert(
            "count".to_string(),
            MoveInfo {
                name: "count".to_string(),
                moved_at: None,
                is_copy: true,
                ty: TypeId::INT32,
            },
        );

        // Simulate passing count to a function (copy type)
        bc.record_move("count", TypeId::INT32, &span_at(1, 10));
        // Should still be usable
        bc.check_use("count", &span_at(2, 5));
        // Pass again
        bc.record_move("count", TypeId::INT32, &span_at(3, 10));
        bc.check_use("count", &span_at(4, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── Reference creation and usage ──────────────────────────

    #[test]
    fn reference_creation_and_usage() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("item");
        // Create immutable reference — should be fine
        bc.check_borrow_conflicts(&place, false, &span_at(1, 5));
        bc.record_borrow(place.clone(), false, &span_at(1, 5));
        // Create another immutable reference — still fine
        bc.check_borrow_conflicts(&place, false, &span_at(2, 5));
        bc.record_borrow(place.clone(), false, &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 0);
    }

    // ── E4005: reference escapes scope ────────────────────────

    #[test]
    fn reference_escapes_scope() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.check_reference_escape("local", 3, 1, &span_at(5, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4005");
        assert!(errors[0].message.contains("reference to `local` escapes its scope"));
    }

    // ── E4006: device transfer ────────────────────────────────

    #[test]
    fn device_transfer_error() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.check_device_transfer(
            "weights",
            &DeviceContext::Gpu,
            &DeviceContext::Cpu,
            &span_at(10, 5),
        );

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4006");
        assert!(errors[0].message.contains("cannot transfer @gpu tensor `weights`"));
    }

    // ── is_copy_type checks ───────────────────────────────────

    #[test]
    fn is_copy_type_primitives() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let bc = make_checker(&interner, &symbols);

        assert!(bc.is_copy_type(TypeId::INT32));
        assert!(bc.is_copy_type(TypeId::BOOL));
        assert!(bc.is_copy_type(TypeId::FLOAT64));
        assert!(bc.is_copy_type(TypeId::CHAR));
        assert!(bc.is_copy_type(TypeId::UNIT));
        assert!(bc.is_copy_type(TypeId::NEVER));
        // String is NOT copy
        assert!(!bc.is_copy_type(TypeId::STRING));
    }

    // ── Place overlaps ────────────────────────────────────────

    #[test]
    fn place_overlaps_logic() {
        let a = Place::var("x");
        let b = Place::var("x").with_field("f");
        let c = Place::var("y");

        assert!(a.overlaps(&b)); // x overlaps x.f
        assert!(b.overlaps(&a)); // x.f overlaps x
        assert!(!a.overlaps(&c)); // x does not overlap y
    }

    // ── Immutable borrow blocks mutable borrow (symmetric) ───

    #[test]
    fn immutable_borrow_while_mutable_exists() {
        let interner = TypeInterner::new();
        let symbols = SymbolTable::new();
        let mut bc = make_checker(&interner, &symbols);

        bc.enter_scope();
        let place = Place::var("d");
        // Mutable borrow first
        bc.record_borrow(place.clone(), true, &span_at(1, 5));
        // Try immutable borrow
        bc.check_borrow_conflicts(&place, false, &span_at(2, 5));

        let errors = bc.take_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, "E4002");
    }
}
