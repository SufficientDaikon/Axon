// mir/transform/const_fold.rs — Constant folding pass
//
// Evaluates constant expressions at compile time:
// - Binary ops on two constants: 2 + 3 → 5
// - Unary ops on constants: -5 → -5

use crate::mir::{MirProgram, MirStmt, Rvalue, Operand, MirConstant, MirBinOp, MirUnaryOp};
use crate::types::TypeInterner;
use super::MirPass;

pub struct ConstantFolding;

impl MirPass for ConstantFolding {
    fn name(&self) -> &'static str {
        "const_fold"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) {
        for func in &mut program.functions {
            for block in &mut func.basic_blocks {
                for stmt in &mut block.stmts {
                    fold_stmt(stmt);
                }
            }
        }
    }
}

fn fold_stmt(stmt: &mut MirStmt) {
    if let MirStmt::Assign { rvalue, .. } = stmt {
        if let Some(folded) = try_fold_rvalue(rvalue) {
            *rvalue = Rvalue::Use(Operand::Constant(folded));
        }
    }
}

fn try_fold_rvalue(rvalue: &Rvalue) -> Option<MirConstant> {
    match rvalue {
        Rvalue::BinaryOp { op, left, right } => {
            let l = extract_constant(left)?;
            let r = extract_constant(right)?;
            fold_binary(op, &l, &r)
        }
        Rvalue::UnaryOp { op, operand } => {
            let c = extract_constant(operand)?;
            fold_unary(op, &c)
        }
        _ => None,
    }
}

fn extract_constant(op: &Operand) -> Option<MirConstant> {
    match op {
        Operand::Constant(c) => Some(c.clone()),
        _ => None,
    }
}

fn fold_binary(op: &MirBinOp, left: &MirConstant, right: &MirConstant) -> Option<MirConstant> {
    match (left, right) {
        // Int64 arithmetic
        (MirConstant::Int(a), MirConstant::Int(b)) => match op {
            MirBinOp::Add => Some(MirConstant::Int(a.wrapping_add(*b))),
            MirBinOp::Sub => Some(MirConstant::Int(a.wrapping_sub(*b))),
            MirBinOp::Mul => Some(MirConstant::Int(a.wrapping_mul(*b))),
            MirBinOp::Div => {
                if *b == 0 { None } else { Some(MirConstant::Int(a.wrapping_div(*b))) }
            }
            MirBinOp::Mod => {
                if *b == 0 { None } else { Some(MirConstant::Int(a.wrapping_rem(*b))) }
            }
            MirBinOp::Eq => Some(MirConstant::Bool(a == b)),
            MirBinOp::Ne => Some(MirConstant::Bool(a != b)),
            MirBinOp::Lt => Some(MirConstant::Bool(a < b)),
            MirBinOp::Le => Some(MirConstant::Bool(a <= b)),
            MirBinOp::Gt => Some(MirConstant::Bool(a > b)),
            MirBinOp::Ge => Some(MirConstant::Bool(a >= b)),
            _ => None,
        },

        // Float64 arithmetic
        (MirConstant::Float(a), MirConstant::Float(b)) => match op {
            MirBinOp::Add => Some(MirConstant::Float(a + b)),
            MirBinOp::Sub => Some(MirConstant::Float(a - b)),
            MirBinOp::Mul => Some(MirConstant::Float(a * b)),
            MirBinOp::Div => {
                if *b == 0.0 { None } else { Some(MirConstant::Float(a / b)) }
            }
            MirBinOp::Eq => Some(MirConstant::Bool(a == b)),
            MirBinOp::Ne => Some(MirConstant::Bool(a != b)),
            MirBinOp::Lt => Some(MirConstant::Bool(a < b)),
            MirBinOp::Le => Some(MirConstant::Bool(a <= b)),
            MirBinOp::Gt => Some(MirConstant::Bool(a > b)),
            MirBinOp::Ge => Some(MirConstant::Bool(a >= b)),
            _ => None,
        },

        // Boolean logic
        (MirConstant::Bool(a), MirConstant::Bool(b)) => match op {
            MirBinOp::And => Some(MirConstant::Bool(*a && *b)),
            MirBinOp::Or => Some(MirConstant::Bool(*a || *b)),
            MirBinOp::Eq => Some(MirConstant::Bool(a == b)),
            MirBinOp::Ne => Some(MirConstant::Bool(a != b)),
            _ => None,
        },

        _ => None,
    }
}

fn fold_unary(op: &MirUnaryOp, operand: &MirConstant) -> Option<MirConstant> {
    match (op, operand) {
        (MirUnaryOp::Neg, MirConstant::Int(n)) => Some(MirConstant::Int(-n)),
        (MirUnaryOp::Neg, MirConstant::Float(f)) => Some(MirConstant::Float(-f)),
        (MirUnaryOp::Not, MirConstant::Bool(b)) => Some(MirConstant::Bool(!b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::span::Span;
    use crate::types::TypeId;

    fn make_assign(rvalue: Rvalue) -> MirStmt {
        MirStmt::Assign {
            place: Place { local: LocalId(0), projections: vec![] },
            rvalue,
            span: Span::dummy(),
        }
    }

    fn make_binop(op: MirBinOp, left: MirConstant, right: MirConstant) -> Rvalue {
        Rvalue::BinaryOp {
            op,
            left: Operand::Constant(left),
            right: Operand::Constant(right),
        }
    }

    #[test]
    fn test_folds_int_addition() {
        let mut stmt = make_assign(make_binop(MirBinOp::Add, MirConstant::Int(2), MirConstant::Int(3)));
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Int(5))), .. } = stmt {
            // pass
        } else {
            panic!("Expected folded to Int(5), got {:?}", stmt);
        }
    }

    #[test]
    fn test_folds_float_multiplication() {
        let mut stmt = make_assign(make_binop(MirBinOp::Mul, MirConstant::Float(2.0), MirConstant::Float(3.0)));
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Float(f))), .. } = stmt {
            assert!((f - 6.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected folded to Float(6.0)");
        }
    }

    #[test]
    fn test_folds_boolean_and() {
        let mut stmt = make_assign(make_binop(MirBinOp::And, MirConstant::Bool(true), MirConstant::Bool(false)));
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Bool(false))), .. } = stmt {
            // pass
        } else {
            panic!("Expected folded to Bool(false)");
        }
    }

    #[test]
    fn test_no_fold_non_constant() {
        // Place operand (variable) can't be folded
        let rvalue = Rvalue::BinaryOp {
            op: MirBinOp::Add,
            left: Operand::Place(Place { local: LocalId(0), projections: vec![] }),
            right: Operand::Constant(MirConstant::Int(1)),
        };
        let mut stmt = make_assign(rvalue);
        fold_stmt(&mut stmt);
        // Should remain a BinaryOp, not folded
        if let MirStmt::Assign { rvalue: Rvalue::BinaryOp { .. }, .. } = stmt {
            // pass - not folded (correct)
        } else {
            panic!("Should not fold when one operand is a variable");
        }
    }

    #[test]
    fn test_folds_negation() {
        let rvalue = Rvalue::UnaryOp {
            op: MirUnaryOp::Neg,
            operand: Operand::Constant(MirConstant::Int(5)),
        };
        let mut stmt = make_assign(rvalue);
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Int(-5))), .. } = stmt {
            // pass
        } else {
            panic!("Expected folded to Int(-5)");
        }
    }

    #[test]
    fn test_folds_not() {
        let rvalue = Rvalue::UnaryOp {
            op: MirUnaryOp::Not,
            operand: Operand::Constant(MirConstant::Bool(true)),
        };
        let mut stmt = make_assign(rvalue);
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Bool(false))), .. } = stmt {
            // pass
        } else {
            panic!("Expected folded to Bool(false)");
        }
    }

    #[test]
    fn test_no_fold_div_by_zero() {
        let mut stmt = make_assign(make_binop(MirBinOp::Div, MirConstant::Int(10), MirConstant::Int(0)));
        fold_stmt(&mut stmt);
        // Should NOT fold (division by zero)
        if let MirStmt::Assign { rvalue: Rvalue::BinaryOp { .. }, .. } = stmt {
            // pass
        } else {
            panic!("Should not fold division by zero");
        }
    }

    #[test]
    fn test_folds_comparison() {
        let mut stmt = make_assign(make_binop(MirBinOp::Lt, MirConstant::Int(3), MirConstant::Int(5)));
        fold_stmt(&mut stmt);
        if let MirStmt::Assign { rvalue: Rvalue::Use(Operand::Constant(MirConstant::Bool(true))), .. } = stmt {
            // pass
        } else {
            panic!("Expected folded to Bool(true)");
        }
    }
}
