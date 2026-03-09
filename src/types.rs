use serde::Serialize;
use std::fmt;

// ---------------------------------------------------------------------------
// Primitive kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PrimKind {
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
    Bool,
    Char,
    String,
}

impl PrimKind {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            PrimKind::Int8
                | PrimKind::Int16
                | PrimKind::Int32
                | PrimKind::Int64
                | PrimKind::UInt8
                | PrimKind::UInt16
                | PrimKind::UInt32
                | PrimKind::UInt64
                | PrimKind::Float16
                | PrimKind::Float32
                | PrimKind::Float64
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            PrimKind::Int8
                | PrimKind::Int16
                | PrimKind::Int32
                | PrimKind::Int64
                | PrimKind::UInt8
                | PrimKind::UInt16
                | PrimKind::UInt32
                | PrimKind::UInt64
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(
            self,
            PrimKind::Float16 | PrimKind::Float32 | PrimKind::Float64
        )
    }

    pub fn is_copy(&self) -> bool {
        !matches!(self, PrimKind::String)
    }
}

impl fmt::Display for PrimKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimKind::Int8 => write!(f, "i8"),
            PrimKind::Int16 => write!(f, "i16"),
            PrimKind::Int32 => write!(f, "i32"),
            PrimKind::Int64 => write!(f, "i64"),
            PrimKind::UInt8 => write!(f, "u8"),
            PrimKind::UInt16 => write!(f, "u16"),
            PrimKind::UInt32 => write!(f, "u32"),
            PrimKind::UInt64 => write!(f, "u64"),
            PrimKind::Float16 => write!(f, "f16"),
            PrimKind::Float32 => write!(f, "f32"),
            PrimKind::Float64 => write!(f, "f64"),
            PrimKind::Bool => write!(f, "bool"),
            PrimKind::Char => write!(f, "char"),
            PrimKind::String => write!(f, "string"),
        }
    }
}

// ---------------------------------------------------------------------------
// Lightweight handle types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ScopeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct SymbolId(pub u32);

// Well-known TypeId constants
impl TypeId {
    pub const UNIT: TypeId = TypeId(0);
    pub const NEVER: TypeId = TypeId(1);
    pub const ERROR: TypeId = TypeId(2);
    pub const BOOL: TypeId = TypeId(3);
    pub const INT32: TypeId = TypeId(4);
    pub const INT64: TypeId = TypeId(5);
    pub const FLOAT32: TypeId = TypeId(6);
    pub const FLOAT64: TypeId = TypeId(7);
    pub const STRING: TypeId = TypeId(8);
    pub const CHAR: TypeId = TypeId(9);
}

// ---------------------------------------------------------------------------
// Generic variable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct GenericVar {
    pub name: String,
    pub id: u32,
}

// ---------------------------------------------------------------------------
// Shape dimensions (resolved, for the type system)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ShapeDimResolved {
    Known(i64),
    Dynamic,
    Variable(String),
    /// To be resolved by unification.
    Inferred(u32),
}

// ---------------------------------------------------------------------------
// Tensor type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TensorType {
    pub dtype: TypeId,
    pub shape: Vec<ShapeDimResolved>,
}

// ---------------------------------------------------------------------------
// Enum variant type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum EnumVariantType {
    Unit,
    Tuple(Vec<TypeId>),
    Struct(Vec<(String, TypeId)>),
}

// ---------------------------------------------------------------------------
// Core type representation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
    Primitive(PrimKind),
    Tensor(TensorType),
    Tuple(Vec<TypeId>),
    Array { elem: TypeId, size: usize },
    Reference { mutable: bool, inner: TypeId },
    Function { params: Vec<TypeId>, ret: TypeId },
    Struct {
        name: String,
        fields: Vec<(String, TypeId)>,
        generics: Vec<TypeId>,
    },
    Enum {
        name: String,
        variants: Vec<(String, EnumVariantType)>,
        generics: Vec<TypeId>,
    },
    Trait {
        name: String,
        methods: Vec<(String, TypeId)>,
        supertraits: Vec<TypeId>,
    },
    Generic(GenericVar),
    /// Inference variable.
    TypeVar(u32),
    Named { path: Vec<String>, args: Vec<TypeId> },
    Option(TypeId),
    Result(TypeId, TypeId),
    Never,
    Unit,
    /// Poison type for error recovery.
    Error,
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Primitive(p) => p.is_copy(),
            Type::Unit | Type::Never => true,
            _ => false,
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            Type::Primitive(p) => p.is_numeric(),
            Type::Tensor(_) => true,
            _ => false,
        }
    }

    pub fn is_tensor(&self) -> bool {
        matches!(self, Type::Tensor(_))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p),
            Type::Tensor(t) => {
                write!(f, "tensor<{}, [", t.dtype.0)?;
                for (i, dim) in t.shape.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match dim {
                        ShapeDimResolved::Known(n) => write!(f, "{}", n)?,
                        ShapeDimResolved::Dynamic => write!(f, "?")?,
                        ShapeDimResolved::Variable(name) => write!(f, "{}", name)?,
                        ShapeDimResolved::Inferred(id) => write!(f, "?{}", id)?,
                    }
                }
                write!(f, "]>")
            }
            Type::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "#{}", e.0)?;
                }
                write!(f, ")")
            }
            Type::Array { elem, size } => write!(f, "[#{}; {}]", elem.0, size),
            Type::Reference { mutable, inner } => {
                if *mutable {
                    write!(f, "&mut #{}", inner.0)
                } else {
                    write!(f, "&#{}", inner.0)
                }
            }
            Type::Function { params, ret } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "#{}", p.0)?;
                }
                write!(f, ") -> #{}", ret.0)
            }
            Type::Struct { name, .. } => write!(f, "{}", name),
            Type::Enum { name, .. } => write!(f, "{}", name),
            Type::Trait { name, .. } => write!(f, "trait {}", name),
            Type::Generic(g) => write!(f, "{}", g.name),
            Type::TypeVar(id) => write!(f, "?T{}", id),
            Type::Named { path, args } => {
                write!(f, "{}", path.join("::"))?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "#{}", a.0)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Option(inner) => write!(f, "Option<#{}>", inner.0),
            Type::Result(ok, err) => write!(f, "Result<#{}, #{}>", ok.0, err.0),
            Type::Never => write!(f, "!"),
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "<error>"),
        }
    }
}

// ---------------------------------------------------------------------------
// Type interner — arena for type storage
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct TypeInterner {
    types: Vec<Type>,
}

impl TypeInterner {
    pub fn new() -> Self {
        let mut interner = TypeInterner { types: Vec::new() };
        // Pre-intern common types – order must match TypeId constants.
        interner.intern(Type::Unit);                            // TypeId(0)
        interner.intern(Type::Never);                           // TypeId(1)
        interner.intern(Type::Error);                           // TypeId(2)
        interner.intern(Type::Primitive(PrimKind::Bool));       // TypeId(3)
        interner.intern(Type::Primitive(PrimKind::Int32));      // TypeId(4)
        interner.intern(Type::Primitive(PrimKind::Int64));      // TypeId(5)
        interner.intern(Type::Primitive(PrimKind::Float32));    // TypeId(6)
        interner.intern(Type::Primitive(PrimKind::Float64));    // TypeId(7)
        interner.intern(Type::Primitive(PrimKind::String));     // TypeId(8)
        interner.intern(Type::Primitive(PrimKind::Char));       // TypeId(9)
        interner
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        // Check if already interned (for simple types).
        for (i, existing) in self.types.iter().enumerate() {
            if *existing == ty {
                return TypeId(i as u32);
            }
        }
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    /// Find a primitive type's TypeId without mutating.
    pub fn lookup_prim(&self, prim: PrimKind) -> Option<TypeId> {
        let target = Type::Primitive(prim);
        for (i, existing) in self.types.iter().enumerate() {
            if *existing == target {
                return Some(TypeId(i as u32));
            }
        }
        None
    }

    pub fn resolve(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }

    pub fn resolve_mut(&mut self, id: TypeId) -> &mut Type {
        &mut self.types[id.0 as usize]
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interner_pre_interned_count() {
        let interner = TypeInterner::new();
        assert_eq!(interner.len(), 10);
    }

    #[test]
    fn interner_constants_resolve() {
        let interner = TypeInterner::new();
        assert_eq!(*interner.resolve(TypeId::UNIT), Type::Unit);
        assert_eq!(*interner.resolve(TypeId::NEVER), Type::Never);
        assert_eq!(*interner.resolve(TypeId::ERROR), Type::Error);
        assert_eq!(
            *interner.resolve(TypeId::BOOL),
            Type::Primitive(PrimKind::Bool)
        );
        assert_eq!(
            *interner.resolve(TypeId::INT32),
            Type::Primitive(PrimKind::Int32)
        );
        assert_eq!(
            *interner.resolve(TypeId::INT64),
            Type::Primitive(PrimKind::Int64)
        );
        assert_eq!(
            *interner.resolve(TypeId::FLOAT32),
            Type::Primitive(PrimKind::Float32)
        );
        assert_eq!(
            *interner.resolve(TypeId::FLOAT64),
            Type::Primitive(PrimKind::Float64)
        );
        assert_eq!(
            *interner.resolve(TypeId::STRING),
            Type::Primitive(PrimKind::String)
        );
        assert_eq!(
            *interner.resolve(TypeId::CHAR),
            Type::Primitive(PrimKind::Char)
        );
    }

    #[test]
    fn interner_deduplicates() {
        let mut interner = TypeInterner::new();
        let a = interner.intern(Type::Primitive(PrimKind::Int32));
        let b = interner.intern(Type::Primitive(PrimKind::Int32));
        assert_eq!(a, b);
        assert_eq!(a, TypeId::INT32);
        // Length should not have grown.
        assert_eq!(interner.len(), 10);
    }

    #[test]
    fn interner_new_types() {
        let mut interner = TypeInterner::new();
        let tuple_id = interner.intern(Type::Tuple(vec![TypeId::INT32, TypeId::BOOL]));
        assert_eq!(tuple_id, TypeId(10));
        assert_eq!(interner.len(), 11);

        // Interning the same tuple again should return the same id.
        let tuple_id2 = interner.intern(Type::Tuple(vec![TypeId::INT32, TypeId::BOOL]));
        assert_eq!(tuple_id, tuple_id2);
        assert_eq!(interner.len(), 11);
    }

    #[test]
    fn primkind_is_numeric() {
        assert!(PrimKind::Int8.is_numeric());
        assert!(PrimKind::UInt64.is_numeric());
        assert!(PrimKind::Float16.is_numeric());
        assert!(!PrimKind::Bool.is_numeric());
        assert!(!PrimKind::Char.is_numeric());
        assert!(!PrimKind::String.is_numeric());
    }

    #[test]
    fn primkind_is_integer() {
        assert!(PrimKind::Int8.is_integer());
        assert!(PrimKind::Int64.is_integer());
        assert!(PrimKind::UInt8.is_integer());
        assert!(PrimKind::UInt64.is_integer());
        assert!(!PrimKind::Float32.is_integer());
        assert!(!PrimKind::Bool.is_integer());
    }

    #[test]
    fn primkind_is_float() {
        assert!(PrimKind::Float16.is_float());
        assert!(PrimKind::Float32.is_float());
        assert!(PrimKind::Float64.is_float());
        assert!(!PrimKind::Int32.is_float());
        assert!(!PrimKind::Bool.is_float());
    }

    #[test]
    fn primkind_is_copy() {
        assert!(PrimKind::Int32.is_copy());
        assert!(PrimKind::Bool.is_copy());
        assert!(PrimKind::Char.is_copy());
        assert!(!PrimKind::String.is_copy());
    }

    #[test]
    fn type_is_copy() {
        assert!(Type::Unit.is_copy());
        assert!(Type::Never.is_copy());
        assert!(Type::Primitive(PrimKind::Int32).is_copy());
        assert!(!Type::Primitive(PrimKind::String).is_copy());
        assert!(!Type::Tuple(vec![]).is_copy());
    }

    #[test]
    fn type_is_numeric() {
        assert!(Type::Primitive(PrimKind::Float64).is_numeric());
        assert!(Type::Primitive(PrimKind::Int8).is_numeric());
        let tensor = Type::Tensor(TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![ShapeDimResolved::Known(3), ShapeDimResolved::Known(4)],
        });
        assert!(tensor.is_numeric());
        assert!(!Type::Unit.is_numeric());
    }

    #[test]
    fn type_is_tensor() {
        let tensor = Type::Tensor(TensorType {
            dtype: TypeId::FLOAT32,
            shape: vec![ShapeDimResolved::Dynamic],
        });
        assert!(tensor.is_tensor());
        assert!(!Type::Unit.is_tensor());
    }

    #[test]
    fn display_primkind() {
        assert_eq!(format!("{}", PrimKind::Int32), "i32");
        assert_eq!(format!("{}", PrimKind::Float64), "f64");
        assert_eq!(format!("{}", PrimKind::Bool), "bool");
        assert_eq!(format!("{}", PrimKind::String), "string");
    }

    #[test]
    fn display_type() {
        assert_eq!(format!("{}", Type::Unit), "()");
        assert_eq!(format!("{}", Type::Never), "!");
        assert_eq!(format!("{}", Type::Error), "<error>");
        assert_eq!(format!("{}", Type::Primitive(PrimKind::Int32)), "i32");
    }
}
