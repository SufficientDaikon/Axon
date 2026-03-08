// shapes.rs — Tensor shape checker (Phase 3d)

use std::collections::HashMap;

use crate::error::CompileError;
use crate::span::Span;
use crate::types::{ShapeDimResolved, TensorType, Type, TypeId, TypeInterner};

// ---------------------------------------------------------------------------
// Helper: format a shape for error messages
// ---------------------------------------------------------------------------

fn format_shape(shape: &[ShapeDimResolved]) -> String {
    let mut out = String::from("[");
    for (i, dim) in shape.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        match dim {
            ShapeDimResolved::Known(n) => out.push_str(&n.to_string()),
            ShapeDimResolved::Dynamic => out.push('?'),
            ShapeDimResolved::Variable(name) => out.push_str(name),
            ShapeDimResolved::Inferred(id) => {
                out.push_str(&format!("?{}", id));
            }
        }
    }
    out.push(']');
    out
}

// ---------------------------------------------------------------------------
// ShapeChecker
// ---------------------------------------------------------------------------

pub struct ShapeChecker {
    dim_bindings: HashMap<String, i64>,
    next_inferred: u32,
}

impl ShapeChecker {
    pub fn new() -> Self {
        ShapeChecker {
            dim_bindings: HashMap::new(),
            next_inferred: 0,
        }
    }

    /// Create a fresh inferred dimension.
    pub fn fresh_dim(&mut self) -> ShapeDimResolved {
        let id = self.next_inferred;
        self.next_inferred += 1;
        ShapeDimResolved::Inferred(id)
    }

    // -----------------------------------------------------------------------
    // Dimension unification
    // -----------------------------------------------------------------------

    pub fn unify_dim(
        &mut self,
        a: &ShapeDimResolved,
        b: &ShapeDimResolved,
        span: &Span,
    ) -> Result<ShapeDimResolved, CompileError> {
        use ShapeDimResolved::*;
        match (a, b) {
            // Dynamic absorbs anything.
            (Dynamic, _) | (_, Dynamic) => Ok(Dynamic),

            // Two known values.
            (Known(x), Known(y)) => {
                if x == y {
                    Ok(Known(*x))
                } else {
                    Err(CompileError::new(
                        "E3001",
                        format!("shape mismatch: expected dimension {}, found {}", x, y),
                        span.clone(),
                    ))
                }
            }

            // Known + Variable: bind the variable.
            (Known(v), Variable(name)) | (Variable(name), Known(v)) => {
                if let Some(&existing) = self.dim_bindings.get(name) {
                    if existing != *v {
                        return Err(CompileError::new(
                            "E3001",
                            format!(
                                "shape mismatch: variable {} was bound to {}, but found {}",
                                name, existing, v
                            ),
                            span.clone(),
                        ));
                    }
                } else {
                    self.dim_bindings.insert(name.clone(), *v);
                }
                Ok(Known(*v))
            }

            // Two variables with the same name.
            (Variable(n), Variable(m)) if n == m => Ok(Variable(n.clone())),

            // Two different variables: bind one to the other, keep the first.
            (Variable(n), Variable(m)) => {
                // If either has a binding, try to reconcile.
                match (self.dim_bindings.get(n).copied(), self.dim_bindings.get(m).copied()) {
                    (Some(vn), Some(vm)) if vn != vm => {
                        return Err(CompileError::new(
                            "E3001",
                            format!(
                                "shape mismatch: {} = {} but {} = {}",
                                n, vn, m, vm
                            ),
                            span.clone(),
                        ));
                    }
                    (Some(v), _) => {
                        self.dim_bindings.insert(m.clone(), v);
                    }
                    (_, Some(v)) => {
                        self.dim_bindings.insert(n.clone(), v);
                    }
                    _ => {} // Both unbound — leave symbolic.
                }
                Ok(Variable(n.clone()))
            }

            // Inferred + concrete: resolve to concrete.
            (Inferred(_), other) | (other, Inferred(_)) => Ok(other.clone()),
        }
    }

    // -----------------------------------------------------------------------
    // Shape unification (same rank, per-dim)
    // -----------------------------------------------------------------------

    pub fn unify_shapes(
        &mut self,
        a: &[ShapeDimResolved],
        b: &[ShapeDimResolved],
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        if a.len() != b.len() {
            return Err(CompileError::new(
                "E3001",
                format!(
                    "shape rank mismatch: {} vs {}",
                    format_shape(a),
                    format_shape(b)
                ),
                span.clone(),
            )
            .with_note(format!("left rank is {}, right rank is {}", a.len(), b.len())));
        }
        a.iter()
            .zip(b.iter())
            .map(|(da, db)| self.unify_dim(da, db, span))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Matmul
    // -----------------------------------------------------------------------

    pub fn check_matmul(
        &mut self,
        left: &TensorType,
        right: &TensorType,
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        let lrank = left.shape.len();
        let rrank = right.shape.len();

        if lrank < 2 || rrank < 2 {
            return Err(CompileError::new(
                "E3002",
                format!(
                    "matmul requires at least rank-2 tensors, got {} and {}",
                    format_shape(&left.shape),
                    format_shape(&right.shape)
                ),
                span.clone(),
            ));
        }

        // Inner dimensions: left[..., M, K] @ right[..., K, N]
        let left_k = &left.shape[lrank - 1];
        let right_k = &right.shape[rrank - 2];

        let _k = self.unify_dim(left_k, right_k, span).map_err(|_| {
            CompileError::new(
                "E3002",
                format!(
                    "matmul inner dimension mismatch: left K = {}, right K = {}",
                    dim_str(left_k),
                    dim_str(right_k)
                ),
                span.clone(),
            )
            .with_suggestion("ensure the inner dimensions of the matmul operands match")
        })?;

        let m = left.shape[lrank - 2].clone();
        let n = right.shape[rrank - 1].clone();

        // Batch dimensions.
        let left_batch = &left.shape[..lrank - 2];
        let right_batch = &right.shape[..rrank - 2];

        let batch = if left_batch.is_empty() && right_batch.is_empty() {
            vec![]
        } else {
            self.check_broadcast(left_batch, right_batch, span)?
        };

        let mut result = batch;
        result.push(m);
        result.push(n);
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Elementwise
    // -----------------------------------------------------------------------

    pub fn check_elementwise(
        &mut self,
        left: &TensorType,
        right: &TensorType,
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        self.check_broadcast(&left.shape, &right.shape, span)
    }

    // -----------------------------------------------------------------------
    // Broadcast (numpy rules)
    // -----------------------------------------------------------------------

    pub fn check_broadcast(
        &mut self,
        a: &[ShapeDimResolved],
        b: &[ShapeDimResolved],
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        let max_rank = a.len().max(b.len());
        let mut result = Vec::with_capacity(max_rank);

        for i in 0..max_rank {
            let da = if i < max_rank - a.len() {
                &ShapeDimResolved::Known(1)
            } else {
                &a[i - (max_rank - a.len())]
            };
            let db = if i < max_rank - b.len() {
                &ShapeDimResolved::Known(1)
            } else {
                &b[i - (max_rank - b.len())]
            };

            let unified = self.broadcast_dim(da, db, span).map_err(|_| {
                CompileError::new(
                    "E3003",
                    format!(
                        "cannot broadcast shapes {} and {}",
                        format_shape(a),
                        format_shape(b)
                    ),
                    span.clone(),
                )
                .with_note(format!("dimension {} is incompatible", i))
            })?;
            result.push(unified);
        }

        Ok(result)
    }

    /// Broadcast a single dimension pair.
    fn broadcast_dim(
        &mut self,
        a: &ShapeDimResolved,
        b: &ShapeDimResolved,
        span: &Span,
    ) -> Result<ShapeDimResolved, CompileError> {
        use ShapeDimResolved::*;
        match (a, b) {
            (Dynamic, _) | (_, Dynamic) => Ok(Dynamic),

            (Known(1), other) | (other, Known(1)) => Ok(other.clone()),

            (Known(x), Known(y)) if x == y => Ok(Known(*x)),

            (Known(x), Known(y)) => Err(CompileError::new(
                "E3003",
                format!("cannot broadcast dimensions {} and {}", x, y),
                span.clone(),
            )),

            // Variable dims: attempt unification.
            _ => self.unify_dim(a, b, span),
        }
    }

    // -----------------------------------------------------------------------
    // Reshape
    // -----------------------------------------------------------------------

    pub fn check_reshape(
        &self,
        from: &[ShapeDimResolved],
        to: &[ShapeDimResolved],
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        let from_product = static_product(from);
        let to_product = static_product(to);

        if let (Some(fp), Some(tp)) = (from_product, to_product) {
            if fp != tp {
                return Err(CompileError::new(
                    "E3004",
                    format!(
                        "reshape element count mismatch: source has {} elements, target has {}",
                        fp, tp
                    ),
                    span.clone(),
                )
                .with_note(format!(
                    "source shape {} → target shape {}",
                    format_shape(from),
                    format_shape(to)
                )));
            }
        }
        // If we can't statically verify (Dynamic/Variable dims), defer to runtime.
        Ok(to.to_vec())
    }

    // -----------------------------------------------------------------------
    // Transpose
    // -----------------------------------------------------------------------

    pub fn check_transpose(
        &self,
        shape: &[ShapeDimResolved],
        span: &Span,
    ) -> Result<Vec<ShapeDimResolved>, CompileError> {
        if shape.len() < 2 {
            return Err(CompileError::new(
                "E3001",
                format!(
                    "transpose requires at least rank-2 tensor, got {}",
                    format_shape(shape)
                ),
                span.clone(),
            ));
        }
        let mut result = shape.to_vec();
        let len = result.len();
        result.swap(len - 2, len - 1);
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Validate dtype
    // -----------------------------------------------------------------------

    pub fn validate_dtype(
        interner: &TypeInterner,
        dtype: TypeId,
        span: &Span,
    ) -> Result<(), CompileError> {
        let ty = interner.resolve(dtype);
        match ty {
            Type::Primitive(p) if p.is_numeric() => Ok(()),
            _ => Err(CompileError::new(
                "E3005",
                format!("invalid tensor dtype: {} is not a numeric type", ty),
                span.clone(),
            )
            .with_suggestion("use a numeric type such as f32, f64, i32, etc.")),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dim_str(dim: &ShapeDimResolved) -> String {
    match dim {
        ShapeDimResolved::Known(n) => n.to_string(),
        ShapeDimResolved::Dynamic => "?".to_string(),
        ShapeDimResolved::Variable(name) => name.clone(),
        ShapeDimResolved::Inferred(id) => format!("?{}", id),
    }
}

/// Compute the static product of all Known dims.  Returns `None` if any dim
/// is not statically known.
fn static_product(shape: &[ShapeDimResolved]) -> Option<i64> {
    let mut product: i64 = 1;
    for dim in shape {
        match dim {
            ShapeDimResolved::Known(n) => {
                product = product.checked_mul(*n)?;
            }
            _ => return None,
        }
    }
    Some(product)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use crate::types::{ShapeDimResolved::*, TensorType, TypeId, TypeInterner};

    fn span() -> Span {
        Span::dummy()
    }

    // -- unify_dim -----------------------------------------------------------

    #[test]
    fn unify_known_equal() {
        let mut sc = ShapeChecker::new();
        let r = sc.unify_dim(&Known(128), &Known(128), &span()).unwrap();
        assert_eq!(r, Known(128));
    }

    #[test]
    fn unify_known_mismatch() {
        let mut sc = ShapeChecker::new();
        let r = sc.unify_dim(&Known(128), &Known(256), &span());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().error_code, "E3001");
    }

    #[test]
    fn unify_variable_with_known_binds() {
        let mut sc = ShapeChecker::new();
        let r = sc
            .unify_dim(&Variable("N".into()), &Known(128), &span())
            .unwrap();
        assert_eq!(r, Known(128));
        assert_eq!(sc.dim_bindings.get("N"), Some(&128));
    }

    #[test]
    fn unify_dynamic_absorbs() {
        let mut sc = ShapeChecker::new();
        let r = sc.unify_dim(&Dynamic, &Known(128), &span()).unwrap();
        assert_eq!(r, Dynamic);
    }

    #[test]
    fn unify_inferred_resolves() {
        let mut sc = ShapeChecker::new();
        let r = sc.unify_dim(&Inferred(0), &Known(64), &span()).unwrap();
        assert_eq!(r, Known(64));
    }

    #[test]
    fn unify_two_inferred() {
        let mut sc = ShapeChecker::new();
        let r = sc.unify_dim(&Inferred(0), &Inferred(1), &span()).unwrap();
        // Should keep one of them.
        assert!(matches!(r, Inferred(_)));
    }

    #[test]
    fn unify_variable_same_name() {
        let mut sc = ShapeChecker::new();
        let r = sc
            .unify_dim(&Variable("N".into()), &Variable("N".into()), &span())
            .unwrap();
        assert_eq!(r, Variable("N".into()));
    }

    // -- matmul --------------------------------------------------------------

    #[test]
    fn matmul_2d_ok() {
        let mut sc = ShapeChecker::new();
        let left = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(3), Known(4)],
        };
        let right = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(4), Known(5)],
        };
        let r = sc.check_matmul(&left, &right, &span()).unwrap();
        assert_eq!(r, vec![Known(3), Known(5)]);
    }

    #[test]
    fn matmul_inner_dim_mismatch() {
        let mut sc = ShapeChecker::new();
        let left = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(3), Known(4)],
        };
        let right = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(7), Known(5)],
        };
        let r = sc.check_matmul(&left, &right, &span());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().error_code, "E3002");
    }

    #[test]
    fn matmul_batched() {
        let mut sc = ShapeChecker::new();
        let left = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(2), Known(3), Known(4)],
        };
        let right = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(2), Known(4), Known(5)],
        };
        let r = sc.check_matmul(&left, &right, &span()).unwrap();
        assert_eq!(r, vec![Known(2), Known(3), Known(5)]);
    }

    // -- elementwise / broadcast ---------------------------------------------

    #[test]
    fn elementwise_same_shapes() {
        let mut sc = ShapeChecker::new();
        let left = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(3), Known(4)],
        };
        let right = TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![Known(3), Known(4)],
        };
        let r = sc.check_elementwise(&left, &right, &span()).unwrap();
        assert_eq!(r, vec![Known(3), Known(4)]);
    }

    #[test]
    fn broadcast_one_n_with_m_n() {
        let mut sc = ShapeChecker::new();
        let a = vec![Known(1), Known(4)];
        let b = vec![Known(3), Known(4)];
        let r = sc.check_broadcast(&a, &b, &span()).unwrap();
        assert_eq!(r, vec![Known(3), Known(4)]);
    }

    #[test]
    fn broadcast_lower_rank() {
        let mut sc = ShapeChecker::new();
        let a = vec![Known(3)];
        let b = vec![Known(2), Known(3)];
        let r = sc.check_broadcast(&a, &b, &span()).unwrap();
        assert_eq!(r, vec![Known(2), Known(3)]);
    }

    #[test]
    fn broadcast_incompatible() {
        let mut sc = ShapeChecker::new();
        let a = vec![Known(3)];
        let b = vec![Known(4)];
        let r = sc.check_broadcast(&a, &b, &span());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().error_code, "E3003");
    }

    // -- reshape -------------------------------------------------------------

    #[test]
    fn reshape_ok() {
        let sc = ShapeChecker::new();
        let from = vec![Known(2), Known(3)];
        let to = vec![Known(6)];
        let r = sc.check_reshape(&from, &to, &span()).unwrap();
        assert_eq!(r, vec![Known(6)]);
    }

    #[test]
    fn reshape_mismatch() {
        let sc = ShapeChecker::new();
        let from = vec![Known(2), Known(3)];
        let to = vec![Known(5)];
        let r = sc.check_reshape(&from, &to, &span());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().error_code, "E3004");
    }

    // -- transpose -----------------------------------------------------------

    #[test]
    fn transpose_2d() {
        let sc = ShapeChecker::new();
        let shape = vec![Known(3), Known(4)];
        let r = sc.check_transpose(&shape, &span()).unwrap();
        assert_eq!(r, vec![Known(4), Known(3)]);
    }

    // -- validate_dtype ------------------------------------------------------

    #[test]
    fn validate_dtype_float32_ok() {
        let interner = TypeInterner::new();
        let r = ShapeChecker::validate_dtype(&interner, TypeId::FLOAT32, &span());
        assert!(r.is_ok());
    }

    #[test]
    fn validate_dtype_bool_fails() {
        let interner = TypeInterner::new();
        let r = ShapeChecker::validate_dtype(&interner, TypeId::BOOL, &span());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().error_code, "E3005");
    }

    // -- fresh_dim -----------------------------------------------------------

    #[test]
    fn fresh_dim_increments() {
        let mut sc = ShapeChecker::new();
        let d0 = sc.fresh_dim();
        let d1 = sc.fresh_dim();
        assert_eq!(d0, Inferred(0));
        assert_eq!(d1, Inferred(1));
    }

    // -- format_shape --------------------------------------------------------

    #[test]
    fn format_shape_display() {
        let shape = vec![Known(128), Dynamic, Variable("N".into())];
        assert_eq!(format_shape(&shape), "[128, ?, N]");
    }

    // -- reshape with dynamic dims defers ------------------------------------

    #[test]
    fn reshape_with_dynamic_defers() {
        let sc = ShapeChecker::new();
        let from = vec![Dynamic, Known(3)];
        let to = vec![Known(6)];
        let r = sc.check_reshape(&from, &to, &span()).unwrap();
        assert_eq!(r, vec![Known(6)]);
    }
}
