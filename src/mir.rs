// mir.rs — Mid-level Intermediate Representation (Phase 4a)
//
// Lowers the Typed AST (TAST) into a control-flow-graph-based IR
// where each function is a collection of basic blocks connected by
// explicit terminators (goto, switch, return, call).

use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

use crate::ast::{AssignOp, Attribute, BinOp, Literal, UnaryOp};
use crate::span::Span;
use crate::tast::*;
use crate::types::{PrimKind, Type, TypeId, TypeInterner};

// ═══════════════════════════════════════════════════════════════
// ID Types
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct BlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct LocalId(pub u32);

// ═══════════════════════════════════════════════════════════════
// Core MIR Structures
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    pub statics: Vec<MirStatic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MirFunction {
    pub name: String,
    pub mangled_name: String,
    pub params: Vec<MirLocal>,
    pub return_ty: TypeId,
    pub locals: Vec<MirLocal>,
    pub basic_blocks: Vec<MirBasicBlock>,
    pub attributes: Vec<Attribute>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct MirLocal {
    pub id: LocalId,
    pub name: Option<String>,
    pub ty: TypeId,
    pub mutable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MirStatic {
    pub name: String,
    pub ty: TypeId,
    pub initializer: Option<MirConstant>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MirBasicBlock {
    pub id: BlockId,
    pub stmts: Vec<MirStmt>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Serialize)]
pub enum MirStmt {
    Assign {
        place: Place,
        rvalue: Rvalue,
        span: Span,
    },
    Drop {
        place: Place,
        span: Span,
    },
    StorageLive {
        local: LocalId,
    },
    StorageDead {
        local: LocalId,
    },
    Nop,
}

#[derive(Debug, Clone, Serialize)]
pub enum Terminator {
    Goto { target: BlockId },
    SwitchInt {
        value: Operand,
        targets: Vec<(i64, BlockId)>,
        otherwise: BlockId,
    },
    Return,
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Place,
        target: BlockId,
    },
    Assert {
        cond: Operand,
        msg: String,
        target: BlockId,
    },
    Unreachable,
}

// ═══════════════════════════════════════════════════════════════
// Places, Operands, Rvalues
// ═══════════════════════════════════════════════════════════════

/// An lvalue: local variable + optional projections (field access, index, deref).
#[derive(Debug, Clone, Serialize)]
pub struct Place {
    pub local: LocalId,
    pub projections: Vec<Projection>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Projection {
    Field(u32),
    Index(Operand),
    Deref,
}

#[derive(Debug, Clone, Serialize)]
pub enum Operand {
    Place(Place),
    Constant(MirConstant),
}

#[derive(Debug, Clone, Serialize)]
pub enum MirConstant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Unit,
}

#[derive(Debug, Clone, Serialize)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp {
        op: MirBinOp,
        left: Operand,
        right: Operand,
    },
    UnaryOp {
        op: MirUnaryOp,
        operand: Operand,
    },
    Ref {
        mutable: bool,
        place: Place,
    },
    Aggregate {
        kind: AggregateKind,
        fields: Vec<Operand>,
    },
    Cast {
        operand: Operand,
        target_ty: TypeId,
    },
    Len {
        place: Place,
    },
    TensorOp {
        kind: TensorOpKind,
        operands: Vec<Operand>,
    },
    Discriminant {
        place: Place,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum MirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Serialize)]
pub enum MirUnaryOp {
    Neg,
    Not,
    Deref,
}

#[derive(Debug, Clone, Serialize)]
pub enum AggregateKind {
    Tuple,
    Array,
    Struct(String),
    Enum(String, u32),
}

#[derive(Debug, Clone, Serialize)]
pub enum TensorOpKind {
    MatMul,
    Add,
    Sub,
    Mul,
    Div,
    Reshape(Vec<i64>),
    Transpose,
    Broadcast,
}

// ═══════════════════════════════════════════════════════════════
// MIR Builder — Lowers TAST to MIR
// ═══════════════════════════════════════════════════════════════

pub struct MirBuilder<'a> {
    interner: &'a TypeInterner,
    functions: Vec<MirFunction>,
    statics: Vec<MirStatic>,
    // Per-function state
    current_blocks: Vec<MirBasicBlock>,
    current_locals: Vec<MirLocal>,
    current_block: BlockId,
    next_local: u32,
    next_block: u32,
    local_map: HashMap<String, LocalId>,
    /// Maps source function names to their mangled/IR names.
    function_map: HashMap<String, String>,
}

impl<'a> MirBuilder<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        MirBuilder {
            interner,
            functions: Vec::new(),
            statics: Vec::new(),
            current_blocks: Vec::new(),
            current_locals: Vec::new(),
            current_block: BlockId(0),
            next_local: 0,
            next_block: 0,
            local_map: HashMap::new(),
            function_map: HashMap::new(),
        }
    }

    pub fn build(&mut self, program: &TypedProgram) -> MirProgram {
        // First pass: collect all function names and build function_map
        self.populate_function_map(program);

        for item in &program.items {
            self.lower_item(item);
        }
        MirProgram {
            functions: self.functions.clone(),
            statics: self.statics.clone(),
        }
    }

    fn populate_function_map(&mut self, program: &TypedProgram) {
        for item in &program.items {
            self.collect_function_names(item);
        }
        // Note: print/println are handled specially in lower_fn_call
        // and do not need function_map entries.
    }

    fn collect_function_names(&mut self, item: &TypedItem) {
        match &item.kind {
            TypedItemKind::Function(decl) => {
                let mangled = if decl.name == "main" {
                    "_axon_main".to_string()
                } else {
                    decl.name.clone()
                };
                self.function_map.insert(decl.name.clone(), mangled);
            }
            TypedItemKind::Impl(impl_block) => {
                for sub_item in &impl_block.items {
                    self.collect_function_names(sub_item);
                }
            }
            TypedItemKind::Trait(trait_decl) => {
                for sub_item in &trait_decl.items {
                    self.collect_function_names(sub_item);
                }
            }
            _ => {}
        }
    }

    // ── Block management ───────────────────────────────────────

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        self.current_blocks.push(MirBasicBlock {
            id,
            stmts: Vec::new(),
            terminator: Terminator::Unreachable, // placeholder
        });
        id
    }

    fn switch_to_block(&mut self, block: BlockId) {
        self.current_block = block;
    }

    fn current_block_mut(&mut self) -> &mut MirBasicBlock {
        let idx = self.current_block.0 as usize;
        &mut self.current_blocks[idx]
    }

    fn emit_stmt(&mut self, stmt: MirStmt) {
        let idx = self.current_block.0 as usize;
        self.current_blocks[idx].stmts.push(stmt);
    }

    fn set_terminator(&mut self, term: Terminator) {
        let idx = self.current_block.0 as usize;
        self.current_blocks[idx].terminator = term;
    }

    // ── Local management ───────────────────────────────────────

    fn new_local(&mut self, name: Option<String>, ty: TypeId, mutable: bool) -> LocalId {
        let id = LocalId(self.next_local);
        self.next_local += 1;
        if let Some(ref n) = name {
            self.local_map.insert(n.clone(), id);
        }
        self.current_locals.push(MirLocal {
            id,
            name,
            ty,
            mutable,
        });
        id
    }

    fn new_temp(&mut self, ty: TypeId) -> LocalId {
        self.new_local(None, ty, false)
    }

    /// Get the type of a MIR operand from the current function's locals.
    fn operand_type(&self, operand: &Operand) -> TypeId {
        match operand {
            Operand::Place(place) => {
                self.current_locals
                    .iter()
                    .find(|l| l.id == place.local)
                    .map(|l| l.ty)
                    .unwrap_or(TypeId::ERROR)
            }
            Operand::Constant(c) => match c {
                MirConstant::Int(_) => TypeId::INT64,
                MirConstant::Float(_) => TypeId::FLOAT64,
                MirConstant::Bool(_) => TypeId::BOOL,
                MirConstant::Char(_) => TypeId::CHAR,
                MirConstant::String(_) => TypeId::STRING,
                MirConstant::Unit => TypeId::UNIT,
            },
        }
    }

    fn lookup_local(&self, name: &str) -> Option<LocalId> {
        self.local_map.get(name).copied()
    }

    // ── Reset per-function state ───────────────────────────────

    fn reset_function_state(&mut self) {
        self.current_blocks.clear();
        self.current_locals.clear();
        self.current_block = BlockId(0);
        self.next_local = 0;
        self.next_block = 0;
        self.local_map.clear();
    }

    // ── Item lowering ──────────────────────────────────────────

    fn lower_item(&mut self, item: &TypedItem) {
        match &item.kind {
            TypedItemKind::Function(decl) => {
                self.lower_function(decl, &item.attributes);
            }
            TypedItemKind::Impl(impl_block) => {
                for sub_item in &impl_block.items {
                    self.lower_item(sub_item);
                }
            }
            TypedItemKind::Trait(trait_decl) => {
                for sub_item in &trait_decl.items {
                    self.lower_item(sub_item);
                }
            }
            // Struct/Enum declarations don't generate MIR directly
            _ => {}
        }
    }

    fn lower_function(&mut self, decl: &TypedFnDecl, attrs: &[Attribute]) {
        self.reset_function_state();

        // _0 is the return place
        let ret_local = self.new_local(None, decl.return_type, false);

        // Create locals for parameters
        let mut param_locals = Vec::new();
        for param in &decl.params {
            let local = self.new_local(Some(param.name.clone()), param.ty, false);
            param_locals.push(MirLocal {
                id: local,
                name: Some(param.name.clone()),
                ty: param.ty,
                mutable: false,
            });
        }

        // Create entry block
        let entry = self.new_block();
        self.switch_to_block(entry);

        // Lower body if present
        if let Some(body) = &decl.body {
            let result = self.lower_block(body);
            // Assign result to return place
            self.emit_stmt(MirStmt::Assign {
                place: Place {
                    local: ret_local,
                    projections: Vec::new(),
                },
                rvalue: Rvalue::Use(result),
                span: decl.span.clone(),
            });
            // Emit drops for non-Copy locals
            self.emit_drops_for_scope();
            self.set_terminator(Terminator::Return);
        } else {
            self.set_terminator(Terminator::Return);
        }

        let mir_fn = MirFunction {
            name: decl.name.clone(),
            mangled_name: self.function_map.get(&decl.name).cloned().unwrap_or_else(|| decl.name.clone()),
            params: param_locals,
            return_ty: decl.return_type,
            locals: self.current_locals.clone(),
            basic_blocks: self.current_blocks.clone(),
            attributes: attrs.to_vec(),
            span: decl.span.clone(),
        };
        self.functions.push(mir_fn);
    }

    // ── Block & statement lowering ─────────────────────────────

    fn lower_block(&mut self, block: &TypedBlock) -> Operand {
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        if let Some(tail) = &block.tail_expr {
            self.lower_expr(tail)
        } else {
            Operand::Constant(MirConstant::Unit)
        }
    }

    fn lower_stmt(&mut self, stmt: &TypedStmt) {
        match &stmt.kind {
            TypedStmtKind::Let {
                name,
                mutable,
                ty,
                initializer,
            } => {
                let local = self.new_local(Some(name.clone()), *ty, *mutable);
                self.emit_stmt(MirStmt::StorageLive { local });
                if let Some(init) = initializer {
                    let val = self.lower_expr(init);
                    self.emit_stmt(MirStmt::Assign {
                        place: Place {
                            local,
                            projections: Vec::new(),
                        },
                        rvalue: Rvalue::Use(val),
                        span: stmt.span.clone(),
                    });
                }
            }
            TypedStmtKind::Expr(expr) => {
                let _ = self.lower_expr(expr);
            }
            TypedStmtKind::Return(opt_expr) => {
                let val = if let Some(expr) = opt_expr {
                    self.lower_expr(expr)
                } else {
                    Operand::Constant(MirConstant::Unit)
                };
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local: LocalId(0), // return place
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Use(val),
                    span: stmt.span.clone(),
                });
                self.emit_drops_for_scope();
                self.set_terminator(Terminator::Return);
                // Start unreachable block after return
                let unreachable = self.new_block();
                self.switch_to_block(unreachable);
            }
            TypedStmtKind::While { condition, body } => {
                self.lower_while(condition, body);
            }
            TypedStmtKind::For {
                pattern,
                iterator,
                body,
            } => {
                self.lower_for(pattern, iterator, body);
            }
            TypedStmtKind::Item(item) => {
                self.lower_item(item);
            }
        }
    }

    // ── Expression lowering ────────────────────────────────────

    fn lower_expr(&mut self, expr: &TypedExpr) -> Operand {
        match &expr.kind {
            TypedExprKind::Literal(lit) => match lit {
                Literal::Int(v) => Operand::Constant(MirConstant::Int(*v)),
                Literal::Float(v) => Operand::Constant(MirConstant::Float(*v)),
                Literal::Bool(v) => Operand::Constant(MirConstant::Bool(*v)),
                Literal::Char(v) => Operand::Constant(MirConstant::Char(*v)),
                Literal::String(v) => Operand::Constant(MirConstant::String(v.clone())),
            },
            TypedExprKind::Identifier(name) => {
                if let Some(local) = self.lookup_local(name) {
                    Operand::Place(Place {
                        local,
                        projections: Vec::new(),
                    })
                } else {
                    // Unresolved identifier — may be a function name; emit as constant placeholder
                    Operand::Constant(MirConstant::Unit)
                }
            }
            TypedExprKind::Path(segments) => {
                // Try local lookup on the last segment
                let name = segments.last().map(|s| s.as_str()).unwrap_or("");
                if let Some(local) = self.lookup_local(name) {
                    Operand::Place(Place {
                        local,
                        projections: Vec::new(),
                    })
                } else {
                    Operand::Constant(MirConstant::Unit)
                }
            }
            TypedExprKind::BinaryOp { left, op, right } => {
                self.lower_binary_op(left, op, right, expr.ty)
            }
            TypedExprKind::UnaryOp { op, operand } => {
                self.lower_unary_op(op, operand, expr.ty)
            }
            TypedExprKind::FnCall { function, args } => {
                self.lower_fn_call(function, args, expr.ty)
            }
            TypedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                // Lower method call as a function call with receiver as first arg
                let recv = self.lower_expr(receiver);
                let mut all_args = vec![recv];
                for arg in args {
                    all_args.push(self.lower_expr(arg));
                }
                let dest = self.new_temp(expr.ty);
                let cont = self.new_block();
                self.set_terminator(Terminator::Call {
                    func: Operand::Constant(MirConstant::String(method.clone())),
                    args: all_args,
                    destination: Place {
                        local: dest,
                        projections: Vec::new(),
                    },
                    target: cont,
                });
                self.switch_to_block(cont);
                Operand::Place(Place {
                    local: dest,
                    projections: Vec::new(),
                })
            }
            TypedExprKind::FieldAccess { object, field } => {
                self.lower_field_access(object, field, expr.ty)
            }
            TypedExprKind::Index { object, index } => {
                self.lower_index(object, index, expr.ty)
            }
            TypedExprKind::IfElse {
                condition,
                then_block,
                else_block,
            } => self.lower_if_else(condition, then_block, else_block, expr.ty),
            TypedExprKind::Match { expr: scrutinee, arms } => {
                self.lower_match(scrutinee, arms, expr.ty)
            }
            TypedExprKind::Block(block) => self.lower_block(block),
            TypedExprKind::Reference { mutable, expr: inner } => {
                self.lower_reference(*mutable, inner, expr.ty)
            }
            TypedExprKind::TypeCast {
                expr: inner,
                target_type,
            } => {
                let operand = self.lower_expr(inner);
                let dest = self.new_temp(expr.ty);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local: dest,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Cast {
                        operand,
                        target_ty: *target_type,
                    },
                    span: expr.span.clone(),
                });
                Operand::Place(Place {
                    local: dest,
                    projections: Vec::new(),
                })
            }
            TypedExprKind::Assignment { target, op, value } => {
                let val = self.lower_expr(value);
                let place = self.lower_expr_to_place(target);
                let rvalue = match op {
                    AssignOp::Assign => Rvalue::Use(val),
                    AssignOp::AddAssign => Rvalue::BinaryOp {
                        op: MirBinOp::Add,
                        left: Operand::Place(place.clone()),
                        right: val,
                    },
                    AssignOp::SubAssign => Rvalue::BinaryOp {
                        op: MirBinOp::Sub,
                        left: Operand::Place(place.clone()),
                        right: val,
                    },
                    AssignOp::MulAssign => Rvalue::BinaryOp {
                        op: MirBinOp::Mul,
                        left: Operand::Place(place.clone()),
                        right: val,
                    },
                    AssignOp::DivAssign => Rvalue::BinaryOp {
                        op: MirBinOp::Div,
                        left: Operand::Place(place.clone()),
                        right: val,
                    },
                };
                self.emit_stmt(MirStmt::Assign {
                    place,
                    rvalue,
                    span: expr.span.clone(),
                });
                Operand::Constant(MirConstant::Unit)
            }
            TypedExprKind::Tuple(elems) => {
                let mut fields = Vec::new();
                for elem in elems {
                    fields.push(self.lower_expr(elem));
                }
                let dest = self.new_temp(expr.ty);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local: dest,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Aggregate {
                        kind: AggregateKind::Tuple,
                        fields,
                    },
                    span: expr.span.clone(),
                });
                Operand::Place(Place {
                    local: dest,
                    projections: Vec::new(),
                })
            }
            TypedExprKind::StructLiteral { name, fields } => {
                self.lower_struct_literal(name, fields, expr.ty)
            }
            TypedExprKind::Closure { params, body } => {
                // Lower closure as a simple expression for now
                let _ = params;
                self.lower_expr(body)
            }
            TypedExprKind::ErrorPropagation(inner) => {
                // Simplified: just lower the inner expression
                self.lower_expr(inner)
            }
            TypedExprKind::Range { start, end } => {
                // Lower range as a tuple (start, end)
                let mut fields = Vec::new();
                if let Some(s) = start {
                    fields.push(self.lower_expr(s));
                }
                if let Some(e) = end {
                    fields.push(self.lower_expr(e));
                }
                let dest = self.new_temp(expr.ty);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local: dest,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Aggregate {
                        kind: AggregateKind::Tuple,
                        fields,
                    },
                    span: expr.span.clone(),
                });
                Operand::Place(Place {
                    local: dest,
                    projections: Vec::new(),
                })
            }
        }
    }

    fn lower_expr_to_place(&mut self, expr: &TypedExpr) -> Place {
        match &expr.kind {
            TypedExprKind::Identifier(name) => {
                if let Some(local) = self.lookup_local(name) {
                    Place {
                        local,
                        projections: Vec::new(),
                    }
                } else {
                    let local = self.new_temp(expr.ty);
                    Place {
                        local,
                        projections: Vec::new(),
                    }
                }
            }
            TypedExprKind::FieldAccess { object, field } => {
                let base = self.lower_expr_to_place(object);
                let field_idx = self.resolve_field_index(&object.ty, field);
                let mut projections = base.projections;
                projections.push(Projection::Field(field_idx));
                Place {
                    local: base.local,
                    projections,
                }
            }
            TypedExprKind::Index { object, index } => {
                let base = self.lower_expr_to_place(object);
                let idx_operand = self.lower_expr(index);
                let mut projections = base.projections;
                projections.push(Projection::Index(idx_operand));
                Place {
                    local: base.local,
                    projections,
                }
            }
            _ => {
                // Fallback: evaluate into a temp
                let operand = self.lower_expr(expr);
                let local = self.new_temp(expr.ty);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Use(operand),
                    span: expr.span.clone(),
                });
                Place {
                    local,
                    projections: Vec::new(),
                }
            }
        }
    }

    // ── Specific expression lowering ───────────────────────────

    fn lower_binary_op(
        &mut self,
        left: &TypedExpr,
        op: &BinOp,
        right: &TypedExpr,
        ty: TypeId,
    ) -> Operand {
        let left_op = self.lower_expr(left);
        let right_op = self.lower_expr(right);

        // If the TAST type is ERROR, infer from the operands
        let effective_ty = if ty == TypeId::ERROR {
            // For comparison ops, result is always Bool
            match op {
                BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::LtEq
                | BinOp::Gt | BinOp::GtEq | BinOp::And | BinOp::Or => TypeId::BOOL,
                _ => self.operand_type(&left_op),
            }
        } else {
            ty
        };

        let dest = self.new_temp(effective_ty);

        // Check for tensor operations
        // MatMul (@) is always a tensor op; for other ops, check if the type is tensor
        if *op == BinOp::MatMul || Self::is_tensor_op(op, left.ty, self.interner) {
            let kind = match op {
                BinOp::MatMul => TensorOpKind::MatMul,
                BinOp::Add => TensorOpKind::Add,
                BinOp::Sub => TensorOpKind::Sub,
                BinOp::Mul => TensorOpKind::Mul,
                BinOp::Div => TensorOpKind::Div,
                _ => {
                    // Fall through to scalar op
                    let mir_op = Self::ast_binop_to_mir(op);
                    self.emit_stmt(MirStmt::Assign {
                        place: Place {
                            local: dest,
                            projections: Vec::new(),
                        },
                        rvalue: Rvalue::BinaryOp {
                            op: mir_op,
                            left: left_op,
                            right: right_op,
                        },
                        span: Span::dummy(),
                    });
                    return Operand::Place(Place {
                        local: dest,
                        projections: Vec::new(),
                    });
                }
            };
            self.emit_stmt(MirStmt::Assign {
                place: Place {
                    local: dest,
                    projections: Vec::new(),
                },
                rvalue: Rvalue::TensorOp {
                    kind,
                    operands: vec![left_op, right_op],
                },
                span: Span::dummy(),
            });
        } else {
            let mir_op = Self::ast_binop_to_mir(op);
            self.emit_stmt(MirStmt::Assign {
                place: Place {
                    local: dest,
                    projections: Vec::new(),
                },
                rvalue: Rvalue::BinaryOp {
                    op: mir_op,
                    left: left_op,
                    right: right_op,
                },
                span: Span::dummy(),
            });
        }

        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    fn lower_unary_op(
        &mut self,
        op: &UnaryOp,
        operand: &TypedExpr,
        ty: TypeId,
    ) -> Operand {
        let inner = self.lower_expr(operand);
        let effective_ty = if ty == TypeId::ERROR {
            self.operand_type(&inner)
        } else {
            ty
        };
        let dest = self.new_temp(effective_ty);
        let mir_op = Self::ast_unaryop_to_mir(op);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: dest,
                projections: Vec::new(),
            },
            rvalue: Rvalue::UnaryOp {
                op: mir_op,
                operand: inner,
            },
            span: Span::dummy(),
        });
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    fn lower_fn_call(
        &mut self,
        func: &TypedExpr,
        args: &[TypedExpr],
        ty: TypeId,
    ) -> Operand {
        // ── Special case: print/println builtins ──────────────────────
        if let TypedExprKind::Identifier(name) = &func.kind {
            if name == "print" || name == "println" {
                return self.lower_print_call(name == "println", args);
            }
        }

        // Resolve function name through function_map if it's an identifier
        let func_op = match &func.kind {
            TypedExprKind::Identifier(name) => {
                if let Some(mangled) = self.function_map.get(name) {
                    Operand::Constant(MirConstant::String(mangled.clone()))
                } else if self.lookup_local(name).is_some() {
                    self.lower_expr(func)
                } else {
                    // Unknown function — emit name directly
                    Operand::Constant(MirConstant::String(name.clone()))
                }
            }
            TypedExprKind::Path(segments) => {
                let name = segments.join("::");
                if let Some(mangled) = self.function_map.get(&name) {
                    Operand::Constant(MirConstant::String(mangled.clone()))
                } else {
                    Operand::Constant(MirConstant::String(name))
                }
            }
            _ => self.lower_expr(func),
        };
        let arg_ops: Vec<Operand> = args.iter().map(|a| self.lower_expr(a)).collect();
        let dest = self.new_temp(ty);
        let cont = self.new_block();
        self.set_terminator(Terminator::Call {
            func: func_op,
            args: arg_ops,
            destination: Place {
                local: dest,
                projections: Vec::new(),
            },
            target: cont,
        });
        self.switch_to_block(cont);
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    /// Lower a `print(...)` or `println(...)` call into type-specific runtime calls.
    ///
    /// Maps:
    ///   print(x: Int64)   → call axon_print_i64(x)
    ///   print(x: Float64) → call axon_print_f64(x)
    ///   print(x: Bool)    → call axon_print_bool(x)
    ///   print(x: String)  → call axon_print_str(ptr, len)
    ///   println(x)        → print(x) then call axon_print_newline()
    fn lower_print_call(
        &mut self,
        is_println: bool,
        args: &[TypedExpr],
    ) -> Operand {
        let arg = &args[0];
        let arg_op = self.lower_expr(arg);
        let arg_ty = arg.ty;

        // Determine which runtime function to call based on argument type
        let resolved_ty = self.interner.resolve(arg_ty).clone();
        let (print_fn, call_args) = match &resolved_ty {
            Type::Primitive(PrimKind::Float64) => {
                ("axon_print_f64", vec![arg_op])
            }
            Type::Primitive(PrimKind::Float32) => {
                ("axon_print_f32", vec![arg_op])
            }
            Type::Primitive(PrimKind::Bool) => {
                ("axon_print_bool", vec![arg_op])
            }
            Type::Primitive(PrimKind::String) => {
                // axon_print_str expects (ptr: *i8, len: i64).
                // For string literals, extract the length from the TAST literal node.
                let str_len = if let TypedExprKind::Literal(Literal::String(s)) = &arg.kind {
                    s.len() as i64
                } else {
                    0 // Variable strings: length not yet supported
                };
                ("axon_print_str", vec![arg_op, Operand::Constant(MirConstant::Int(str_len))])
            }
            Type::Primitive(PrimKind::Char) => {
                ("axon_print_char", vec![arg_op])
            }
            Type::Primitive(PrimKind::Int32 | PrimKind::UInt32) => {
                ("axon_print_i32", vec![arg_op])
            }
            // Default: treat as i64 (covers Int64, UInt64, and other integer types)
            _ => {
                ("axon_print_i64", vec![arg_op])
            }
        };

        // Emit the print call
        let dest = self.new_temp(TypeId::UNIT);
        let cont = self.new_block();
        self.set_terminator(Terminator::Call {
            func: Operand::Constant(MirConstant::String(print_fn.to_string())),
            args: call_args,
            destination: Place {
                local: dest,
                projections: Vec::new(),
            },
            target: cont,
        });
        self.switch_to_block(cont);

        // For println, also emit a newline
        if is_println {
            let nl_dest = self.new_temp(TypeId::UNIT);
            let nl_cont = self.new_block();
            self.set_terminator(Terminator::Call {
                func: Operand::Constant(MirConstant::String("axon_print_newline".to_string())),
                args: vec![],
                destination: Place {
                    local: nl_dest,
                    projections: Vec::new(),
                },
                target: nl_cont,
            });
            self.switch_to_block(nl_cont);
        }

        Operand::Constant(MirConstant::Unit)
    }

    fn lower_if_else(
        &mut self,
        cond: &TypedExpr,
        then_block: &TypedBlock,
        else_clause: &Option<TypedElseClause>,
        ty: TypeId,
    ) -> Operand {
        let cond_op = self.lower_expr(cond);
        let result_place = self.new_temp(ty);

        let bb_then = self.new_block();
        let bb_else = self.new_block();
        let bb_merge = self.new_block();

        // SwitchInt: true (1) -> bb_then, otherwise -> bb_else
        self.set_terminator(Terminator::SwitchInt {
            value: cond_op,
            targets: vec![(1, bb_then)],
            otherwise: bb_else,
        });

        // Then block
        self.switch_to_block(bb_then);
        let then_val = self.lower_block(then_block);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: result_place,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Use(then_val),
            span: Span::dummy(),
        });
        self.set_terminator(Terminator::Goto { target: bb_merge });

        // Else block
        self.switch_to_block(bb_else);
        let else_val = if let Some(clause) = else_clause {
            match clause {
                TypedElseClause::ElseBlock(block) => self.lower_block(block),
                TypedElseClause::ElseIf(elif_expr) => self.lower_expr(elif_expr),
            }
        } else {
            Operand::Constant(MirConstant::Unit)
        };
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: result_place,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Use(else_val),
            span: Span::dummy(),
        });
        self.set_terminator(Terminator::Goto { target: bb_merge });

        self.switch_to_block(bb_merge);
        Operand::Place(Place {
            local: result_place,
            projections: Vec::new(),
        })
    }

    fn lower_match(
        &mut self,
        expr: &TypedExpr,
        arms: &[TypedMatchArm],
        ty: TypeId,
    ) -> Operand {
        let scrutinee = self.lower_expr(expr);
        let result_place = self.new_temp(ty);
        let bb_merge = self.new_block();

        // If scrutinee is a place, get discriminant
        let scrutinee_place = match &scrutinee {
            Operand::Place(p) => p.clone(),
            _ => {
                let temp = self.new_temp(expr.ty);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local: temp,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Use(scrutinee),
                    span: expr.span.clone(),
                });
                Place {
                    local: temp,
                    projections: Vec::new(),
                }
            }
        };

        let disc_temp = self.new_temp(TypeId::INT64);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: disc_temp,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Discriminant {
                place: scrutinee_place.clone(),
            },
            span: expr.span.clone(),
        });

        // Build arm blocks
        let mut targets = Vec::new();
        let mut arm_blocks = Vec::new();
        for (i, _arm) in arms.iter().enumerate() {
            let bb_arm = self.new_block();
            arm_blocks.push(bb_arm);
            targets.push((i as i64, bb_arm));
        }

        // Default: last arm or unreachable
        let otherwise = *arm_blocks.last().unwrap_or(&bb_merge);
        // Remove last from targets since it's the otherwise
        if !targets.is_empty() {
            targets.pop();
        }

        self.set_terminator(Terminator::SwitchInt {
            value: Operand::Place(Place {
                local: disc_temp,
                projections: Vec::new(),
            }),
            targets,
            otherwise,
        });

        // Lower each arm body
        for (i, arm) in arms.iter().enumerate() {
            self.switch_to_block(arm_blocks[i]);
            // Bind pattern variables
            self.bind_pattern(&arm.pattern, &scrutinee_place);
            let arm_val = self.lower_expr(&arm.body);
            self.emit_stmt(MirStmt::Assign {
                place: Place {
                    local: result_place,
                    projections: Vec::new(),
                },
                rvalue: Rvalue::Use(arm_val),
                span: arm.span.clone(),
            });
            self.set_terminator(Terminator::Goto { target: bb_merge });
        }

        self.switch_to_block(bb_merge);
        Operand::Place(Place {
            local: result_place,
            projections: Vec::new(),
        })
    }

    fn lower_while(&mut self, cond: &TypedExpr, body: &TypedBlock) {
        let bb_header = self.new_block();
        let bb_body = self.new_block();
        let bb_exit = self.new_block();

        // Goto header
        self.set_terminator(Terminator::Goto { target: bb_header });

        // Header: evaluate condition
        self.switch_to_block(bb_header);
        let cond_op = self.lower_expr(cond);
        self.set_terminator(Terminator::SwitchInt {
            value: cond_op,
            targets: vec![(1, bb_body)],
            otherwise: bb_exit,
        });

        // Body
        self.switch_to_block(bb_body);
        let _ = self.lower_block(body);
        self.set_terminator(Terminator::Goto { target: bb_header });

        // Continue from exit
        self.switch_to_block(bb_exit);
    }

    fn lower_for(&mut self, pattern: &str, iterator: &TypedExpr, body: &TypedBlock) {
        // Lower for loop as: init -> header (call next) -> body -> header -> exit
        let _bb_init = self.current_block;
        let iter_op = self.lower_expr(iterator);
        let iter_local = self.new_temp(iterator.ty);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: iter_local,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Use(iter_op),
            span: Span::dummy(),
        });

        let bb_header = self.new_block();
        let bb_body = self.new_block();
        let bb_exit = self.new_block();

        self.set_terminator(Terminator::Goto { target: bb_header });

        // Header: call iterator.next()
        self.switch_to_block(bb_header);
        let next_result = self.new_temp(TypeId::BOOL);
        let next_block = self.new_block();
        self.set_terminator(Terminator::Call {
            func: Operand::Constant(MirConstant::String("next".into())),
            args: vec![Operand::Place(Place {
                local: iter_local,
                projections: Vec::new(),
            })],
            destination: Place {
                local: next_result,
                projections: Vec::new(),
            },
            target: next_block,
        });

        self.switch_to_block(next_block);
        self.set_terminator(Terminator::SwitchInt {
            value: Operand::Place(Place {
                local: next_result,
                projections: Vec::new(),
            }),
            targets: vec![(1, bb_body)],
            otherwise: bb_exit,
        });

        // Body: bind pattern variable, run body
        self.switch_to_block(bb_body);
        let _pattern_local = self.new_local(Some(pattern.to_string()), TypeId::INT32, false);
        let _ = self.lower_block(body);
        self.set_terminator(Terminator::Goto { target: bb_header });

        // Exit
        self.switch_to_block(bb_exit);
    }

    fn lower_struct_literal(
        &mut self,
        name: &[String],
        fields: &[TypedStructLiteralField],
        ty: TypeId,
    ) -> Operand {
        let struct_name = name.join("::");
        let mut field_ops = Vec::new();
        for field in fields {
            field_ops.push(self.lower_expr(&field.value));
        }
        let dest = self.new_temp(ty);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: dest,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Struct(struct_name),
                fields: field_ops,
            },
            span: Span::dummy(),
        });
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    fn lower_field_access(
        &mut self,
        object: &TypedExpr,
        field: &str,
        ty: TypeId,
    ) -> Operand {
        let base = self.lower_expr_to_place(object);
        let field_idx = self.resolve_field_index(&object.ty, field);
        let dest = self.new_temp(ty);
        let mut projections = base.projections.clone();
        projections.push(Projection::Field(field_idx));
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: dest,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Use(Operand::Place(Place {
                local: base.local,
                projections,
            })),
            span: Span::dummy(),
        });
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    fn lower_index(
        &mut self,
        object: &TypedExpr,
        index: &TypedExpr,
        ty: TypeId,
    ) -> Operand {
        let base = self.lower_expr_to_place(object);
        let idx = self.lower_expr(index);
        let dest = self.new_temp(ty);
        let mut projections = base.projections.clone();
        projections.push(Projection::Index(idx));
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: dest,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Use(Operand::Place(Place {
                local: base.local,
                projections,
            })),
            span: Span::dummy(),
        });
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    fn lower_reference(
        &mut self,
        mutable: bool,
        expr: &TypedExpr,
        ty: TypeId,
    ) -> Operand {
        let place = self.lower_expr_to_place(expr);
        let dest = self.new_temp(ty);
        self.emit_stmt(MirStmt::Assign {
            place: Place {
                local: dest,
                projections: Vec::new(),
            },
            rvalue: Rvalue::Ref {
                mutable,
                place,
            },
            span: Span::dummy(),
        });
        Operand::Place(Place {
            local: dest,
            projections: Vec::new(),
        })
    }

    // ── Helpers ────────────────────────────────────────────────

    fn ast_binop_to_mir(op: &BinOp) -> MirBinOp {
        match op {
            BinOp::Add => MirBinOp::Add,
            BinOp::Sub => MirBinOp::Sub,
            BinOp::Mul => MirBinOp::Mul,
            BinOp::Div => MirBinOp::Div,
            BinOp::Mod => MirBinOp::Mod,
            BinOp::Eq => MirBinOp::Eq,
            BinOp::NotEq => MirBinOp::Ne,
            BinOp::Lt => MirBinOp::Lt,
            BinOp::LtEq => MirBinOp::Le,
            BinOp::Gt => MirBinOp::Gt,
            BinOp::GtEq => MirBinOp::Ge,
            BinOp::And => MirBinOp::And,
            BinOp::Or => MirBinOp::Or,
            BinOp::MatMul => MirBinOp::Mul, // fallback for non-tensor context
        }
    }

    fn ast_unaryop_to_mir(op: &UnaryOp) -> MirUnaryOp {
        match op {
            UnaryOp::Neg => MirUnaryOp::Neg,
            UnaryOp::Not => MirUnaryOp::Not,
            UnaryOp::Ref | UnaryOp::MutRef => MirUnaryOp::Deref, // ref ops handled elsewhere
        }
    }

    fn is_tensor_op(op: &BinOp, ty: TypeId, interner: &TypeInterner) -> bool {
        let resolved = interner.resolve(ty);
        if !resolved.is_tensor() {
            return false;
        }
        matches!(
            op,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::MatMul
        )
    }

    fn resolve_field_index(&self, ty: &TypeId, field: &str) -> u32 {
        let resolved = self.interner.resolve(*ty);
        match resolved {
            Type::Struct { fields, .. } => {
                for (i, (name, _)) in fields.iter().enumerate() {
                    if name == field {
                        return i as u32;
                    }
                }
                0
            }
            _ => 0,
        }
    }

    fn bind_pattern(&mut self, pattern: &TypedPattern, scrutinee: &Place) {
        match &pattern.kind {
            TypedPatternKind::Identifier(name) => {
                let local = self.new_local(Some(name.clone()), pattern.ty, false);
                self.emit_stmt(MirStmt::Assign {
                    place: Place {
                        local,
                        projections: Vec::new(),
                    },
                    rvalue: Rvalue::Use(Operand::Place(scrutinee.clone())),
                    span: pattern.span.clone(),
                });
            }
            TypedPatternKind::Tuple(pats) => {
                for (i, pat) in pats.iter().enumerate() {
                    let mut proj = scrutinee.projections.clone();
                    proj.push(Projection::Field(i as u32));
                    let sub_place = Place {
                        local: scrutinee.local,
                        projections: proj,
                    };
                    self.bind_pattern(pat, &sub_place);
                }
            }
            TypedPatternKind::Struct { fields, .. } => {
                for (i, field_pat) in fields.iter().enumerate() {
                    if let Some(ref pat) = field_pat.pattern {
                        let mut proj = scrutinee.projections.clone();
                        proj.push(Projection::Field(i as u32));
                        let sub_place = Place {
                            local: scrutinee.local,
                            projections: proj,
                        };
                        self.bind_pattern(pat, &sub_place);
                    }
                }
            }
            TypedPatternKind::EnumVariant { fields, .. } => {
                for (i, pat) in fields.iter().enumerate() {
                    let mut proj = scrutinee.projections.clone();
                    proj.push(Projection::Field(i as u32));
                    let sub_place = Place {
                        local: scrutinee.local,
                        projections: proj,
                    };
                    self.bind_pattern(pat, &sub_place);
                }
            }
            // Wildcard, Literal — nothing to bind
            _ => {}
        }
    }

    /// Emit Drop statements for non-Copy locals at scope exit.
    fn emit_drops_for_scope(&mut self) {
        let locals_snapshot: Vec<MirLocal> = self.current_locals.clone();
        for local in locals_snapshot.iter().rev() {
            // Skip return place (_0) and params
            if local.id.0 == 0 {
                continue;
            }
            let ty = self.interner.resolve(local.ty);
            if !ty.is_copy() {
                self.emit_stmt(MirStmt::Drop {
                    place: Place {
                        local: local.id,
                        projections: Vec::new(),
                    },
                    span: Span::dummy(),
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Display — MIR pretty-printing
// ═══════════════════════════════════════════════════════════════

impl fmt::Display for MirProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for func in &self.functions {
            write!(f, "{}", func)?;
            writeln!(f)?;
        }
        for st in &self.statics {
            writeln!(f, "static {}: #{}", st.name, st.ty.0)?;
        }
        Ok(())
    }
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {}() -> #{} {{", self.name, self.return_ty.0)?;
        for local in &self.locals {
            let mutability = if local.mutable { "mut " } else { "" };
            let name = local
                .name
                .as_ref()
                .map(|n| format!(" // {}", n))
                .unwrap_or_default();
            writeln!(
                f,
                "  let {}_{}: #{}{}",
                mutability, local.id.0, local.ty.0, name
            )?;
        }
        writeln!(f)?;
        for bb in &self.basic_blocks {
            write!(f, "{}", bb)?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for MirBasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  bb{}: {{", self.id.0)?;
        for stmt in &self.stmts {
            writeln!(f, "    {}", stmt)?;
        }
        writeln!(f, "    {}", self.terminator)?;
        writeln!(f, "  }}")
    }
}

impl fmt::Display for MirStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirStmt::Assign { place, rvalue, .. } => {
                write!(f, "{} = {}", place, rvalue)
            }
            MirStmt::Drop { place, .. } => {
                write!(f, "drop({})", place)
            }
            MirStmt::StorageLive { local } => {
                write!(f, "StorageLive(_{})", local.0)
            }
            MirStmt::StorageDead { local } => {
                write!(f, "StorageDead(_{})", local.0)
            }
            MirStmt::Nop => write!(f, "nop"),
        }
    }
}

impl fmt::Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terminator::Goto { target } => write!(f, "goto -> bb{}", target.0),
            Terminator::SwitchInt {
                value,
                targets,
                otherwise,
            } => {
                write!(f, "switchInt({}) -> [", value)?;
                for (i, (val, bb)) in targets.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: bb{}", val, bb.0)?;
                }
                if !targets.is_empty() {
                    write!(f, ", ")?;
                }
                write!(f, "otherwise: bb{}]", otherwise.0)
            }
            Terminator::Return => write!(f, "return"),
            Terminator::Call {
                func,
                args,
                destination,
                target,
            } => {
                write!(f, "{} = call {}(", destination, func)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> bb{}", target.0)
            }
            Terminator::Assert {
                cond, msg, target, ..
            } => {
                write!(f, "assert({}, \"{}\") -> bb{}", cond, msg, target.0)
            }
            Terminator::Unreachable => write!(f, "unreachable"),
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_{}", self.local.0)?;
        for proj in &self.projections {
            match proj {
                Projection::Field(idx) => write!(f, ".{}", idx)?,
                Projection::Index(op) => write!(f, "[{}]", op)?,
                Projection::Deref => write!(f, ".*")?,
            }
        }
        Ok(())
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Place(p) => write!(f, "{}", p),
            Operand::Constant(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Display for MirConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirConstant::Int(v) => write!(f, "const {}_i64", v),
            MirConstant::Float(v) => write!(f, "const {}_f64", v),
            MirConstant::Bool(v) => write!(f, "const {}", v),
            MirConstant::Char(v) => write!(f, "const '{}'", v),
            MirConstant::String(v) => write!(f, "const \"{}\"", v),
            MirConstant::Unit => write!(f, "const ()"),
        }
    }
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(op) => write!(f, "{}", op),
            Rvalue::BinaryOp { op, left, right } => {
                write!(f, "{:?}({}, {})", op, left, right)
            }
            Rvalue::UnaryOp { op, operand } => {
                write!(f, "{:?}({})", op, operand)
            }
            Rvalue::Ref { mutable, place } => {
                if *mutable {
                    write!(f, "&mut {}", place)
                } else {
                    write!(f, "&{}", place)
                }
            }
            Rvalue::Aggregate { kind, fields } => {
                write!(f, "{:?}(", kind)?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", field)?;
                }
                write!(f, ")")
            }
            Rvalue::Cast {
                operand,
                target_ty,
            } => {
                write!(f, "{} as #{}", operand, target_ty.0)
            }
            Rvalue::Len { place } => write!(f, "Len({})", place),
            Rvalue::TensorOp { kind, operands } => {
                write!(f, "TensorOp::{:?}(", kind)?;
                for (i, op) in operands.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", op)?;
                }
                write!(f, ")")
            }
            Rvalue::Discriminant { place } => write!(f, "discriminant({})", place),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn build_mir(source: &str) -> MirProgram {
        let (typed_program, errors) = crate::check_source(source, "test.axon");
        // Filter to only hard errors
        let hard_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == crate::error::Severity::Error)
            .collect();
        if !hard_errors.is_empty() {
            panic!(
                "Type-check errors:\n{}",
                hard_errors
                    .iter()
                    .map(|e| format!("  {}", e.message))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        let (checker, _) = crate::typeck::check(source, "test.axon");
        let mut builder = MirBuilder::new(&checker.interner);
        builder.build(&typed_program)
    }

    #[test]
    fn lower_empty_function() {
        let mir = build_mir("fn empty() {}");
        assert_eq!(mir.functions.len(), 1);
        assert_eq!(mir.functions[0].name, "empty");
        assert!(!mir.functions[0].basic_blocks.is_empty());
        // Should have a return terminator in the entry block
        let entry = &mir.functions[0].basic_blocks[0];
        assert!(matches!(entry.terminator, Terminator::Return));
    }

    #[test]
    fn lower_function_with_return_value() {
        let mir = build_mir("fn answer() -> Int64 { return 42; }");
        assert_eq!(mir.functions.len(), 1);
        let func = &mir.functions[0];
        assert_eq!(func.name, "answer");
        // Note: TAST builder currently uses TypeId::ERROR for return types
        // The MIR faithfully preserves what the TAST provides
        let has_return = func.basic_blocks.iter().any(|bb| {
            matches!(bb.terminator, Terminator::Return)
        });
        assert!(has_return, "Expected Return terminator");
        // Should have an assignment with the constant 42
        let has_const_42 = func.basic_blocks.iter().any(|bb| {
            bb.stmts.iter().any(|s| {
                matches!(
                    s,
                    MirStmt::Assign {
                        rvalue: Rvalue::Use(Operand::Constant(MirConstant::Int(42))),
                        ..
                    }
                )
            })
        });
        assert!(has_const_42, "Expected const 42 assignment");
    }

    #[test]
    fn lower_let_binding_and_assignment() {
        let src = "fn test_let() -> Int64 { let x: Int64 = 10; return x; }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        assert_eq!(func.name, "test_let");
        // Should have locals: _0 (return), _1 (x)
        assert!(func.locals.len() >= 2);
        let entry = &func.basic_blocks[0];
        // StorageLive + Assign for let, then assign to _0 and return
        assert!(entry.stmts.len() >= 2);
    }

    #[test]
    fn lower_binary_operations() {
        let src = "fn add(a: Int64, b: Int64) -> Int64 { return a + b; }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        // Should have a BinaryOp assignment somewhere in one of the blocks
        let has_binop = func.basic_blocks.iter().any(|bb| {
            bb.stmts.iter().any(|s| {
                matches!(
                    s,
                    MirStmt::Assign {
                        rvalue: Rvalue::BinaryOp { .. },
                        ..
                    }
                )
            })
        });
        assert!(has_binop, "Expected a BinaryOp assignment in MIR");
    }

    #[test]
    fn lower_if_else_to_switchint() {
        let src = "fn test_if(x: Bool) -> Int64 { if x { return 1; } else { return 2; } }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        // Should have multiple basic blocks
        assert!(
            func.basic_blocks.len() >= 4,
            "Expected at least 4 basic blocks for if/else, got {}",
            func.basic_blocks.len()
        );
        // Should have SwitchInt terminator somewhere
        let has_switch = func.basic_blocks.iter().any(|bb| {
            matches!(bb.terminator, Terminator::SwitchInt { .. })
        });
        assert!(has_switch, "Expected SwitchInt terminator for if/else");
    }

    #[test]
    fn lower_while_loop() {
        let src = "fn test_while() { let mut i: Int64 = 0; while i < 10 { i = i + 1; } }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        // Should have blocks: entry, header, body, exit
        assert!(
            func.basic_blocks.len() >= 4,
            "Expected at least 4 basic blocks for while loop, got {}",
            func.basic_blocks.len()
        );
        // Find the header block with SwitchInt
        let has_switch = func.basic_blocks.iter().any(|bb| {
            matches!(bb.terminator, Terminator::SwitchInt { .. })
        });
        assert!(has_switch, "Expected a SwitchInt terminator in while loop");
        // Find a block that gotos the header (the loop back-edge)
        let header_id = func.basic_blocks.iter().find_map(|bb| {
            if matches!(bb.terminator, Terminator::SwitchInt { .. }) {
                Some(bb.id)
            } else {
                None
            }
        });
        if let Some(hid) = header_id {
            let has_backedge = func.basic_blocks.iter().any(|bb| {
                matches!(bb.terminator, Terminator::Goto { target } if target == hid)
            });
            assert!(has_backedge, "Expected a back-edge to while header");
        }
    }

    #[test]
    fn lower_function_call() {
        let src = "fn callee() -> Int64 { return 42; }\nfn caller() -> Int64 { return callee(); }";
        let mir = build_mir(src);
        assert_eq!(mir.functions.len(), 2);
        let caller = &mir.functions[1];
        // Should have a Call terminator somewhere
        let has_call = caller.basic_blocks.iter().any(|bb| {
            matches!(bb.terminator, Terminator::Call { .. })
        });
        assert!(has_call, "Expected a Call terminator in caller function");
    }

    #[test]
    fn lower_struct_literal_to_aggregate() {
        let src = "struct Point { x: Int64, y: Int64 }\nfn make_point() { let p = Point { x: 1, y: 2 }; }";
        let mir = build_mir(src);
        let func = mir.functions.iter().find(|f| f.name == "make_point").unwrap();
        let has_aggregate = func.basic_blocks.iter().any(|bb| {
            bb.stmts.iter().any(|s| {
                matches!(
                    s,
                    MirStmt::Assign {
                        rvalue: Rvalue::Aggregate {
                            kind: AggregateKind::Struct(_),
                            ..
                        },
                        ..
                    }
                )
            })
        });
        assert!(
            has_aggregate,
            "Expected an Aggregate::Struct rvalue for struct literal"
        );
    }

    #[test]
    fn lower_match_to_switch_chain() {
        let src = "fn test_match(x: Int64) -> Int64 { return match x { 1 => 10, 2 => 20, _ => 0, }; }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        // Should have SwitchInt for the match
        let has_switch = func.basic_blocks.iter().any(|bb| {
            matches!(bb.terminator, Terminator::SwitchInt { .. })
        });
        assert!(has_switch, "Expected SwitchInt for match lowering");
        // Should have multiple arm blocks + merge
        assert!(
            func.basic_blocks.len() >= 4,
            "Expected at least 4 blocks for match with 3 arms, got {}",
            func.basic_blocks.len()
        );
    }

    #[test]
    fn mir_pretty_print() {
        let mir = build_mir("fn hello() { let x: Int64 = 42; }");
        let output = format!("{}", mir);
        assert!(output.contains("fn hello()"), "Should contain function name");
        assert!(output.contains("bb0:"), "Should contain basic block label");
        assert!(output.contains("return"), "Should contain return terminator");
    }

    #[test]
    fn lower_tensor_matmul() {
        let src = "fn matmul(a: Tensor<Float32, [2, 3]>, b: Tensor<Float32, [3, 4]>) {\n\
                    let c = a @ b;\n\
                    }";
        let mir = build_mir(src);
        let func = &mir.functions[0];
        let has_tensor_op = func.basic_blocks.iter().any(|bb| {
            bb.stmts.iter().any(|s| {
                matches!(
                    s,
                    MirStmt::Assign {
                        rvalue: Rvalue::TensorOp {
                            kind: TensorOpKind::MatMul,
                            ..
                        },
                        ..
                    }
                )
            })
        });
        assert!(has_tensor_op, "Expected TensorOp::MatMul for @ operator on tensors");
    }

    #[test]
    fn multiple_functions_in_program() {
        let src = "fn foo() -> Int64 { return 1; }\nfn bar() -> Int64 { return 2; }\nfn baz() -> Int64 { return 3; }";
        let mir = build_mir(src);
        assert_eq!(mir.functions.len(), 3);
        assert_eq!(mir.functions[0].name, "foo");
        assert_eq!(mir.functions[1].name, "bar");
        assert_eq!(mir.functions[2].name, "baz");
    }
}
