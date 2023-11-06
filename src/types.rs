use crate::RpcFunction;
use serde::{Deserialize, Serialize};
use std::{any::type_name, collections::BTreeSet, error::Error, fmt, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Type {
    Nil,
    Int,
    String,
    OneOf(Arc<BTreeSet<Value>>),
    Any,
}

impl Type {
    /// *The* type checker.
    fn check(&self, val: &Value) -> Result<(), TypeMismatchError> {
        match (self, val) {
            // Good type checks
            (Type::Nil, Value::Nil) => (),
            (Type::Int, Value::Int(_)) => (),
            (Type::String, Value::String(_)) => (),
            (Type::OneOf(vals), val) if vals.contains(val) => (),
            (Type::Any, _) => (),

            // All else fails
            _ => Err(TypeMismatchError::rpc_type(self, val))?,
        };
        Ok(())
    }

    pub fn one_of<I, V>(vals: I) -> Type
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        let vals_set = vals.into_iter().map(|v| v.into()).collect();
        Type::OneOf(Arc::new(vals_set))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self {
            Nil => f.write_str("Nil")?,
            Int => f.write_str("Int")?,
            String => f.write_str("String")?,
            OneOf(vals) => {
                f.write_str("OneOf(")?;
                let mut first = true;
                for v in &**vals {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }
                    write!(f, "{v}")?;
                }
                f.write_str(")")?;
            }
            Any => f.write_str("Any")?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatchError(String);

impl TypeMismatchError {
    fn rpc_type(expected_type: &Type, val: &Value) -> Self {
        Self(format!("{val} :/: {expected_type}"))
    }

    fn rust_type<T>(val: &Value) -> Self {
        Self(format!("{val} -/-> {}", type_name::<T>()))
    }
}

impl fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for TypeMismatchError {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Signature {
    pub domain: Type,
    pub range: Type,
}

#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum Value {
    Nil,
    Int(i64),
    String(Arc<String>),
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Nil
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(Arc::new(s))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        s.to_owned().into()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;
        match self {
            Nil => f.write_str("<Nil>"),
            Int(n) => write!(f, "<{n}>"),
            String(s) => write!(f, r#"<"{s}">"#),
        }
    }
}

pub trait InferType {
    fn infer_type() -> Type;
}

pub trait Decode: Sized {
    fn decode(val: Value) -> Option<Self>;
}

pub trait Encode {
    fn encode(val: Self) -> Value;
}

macro_rules! impl_encode_decode {
    ($rust_type:ty, $infer_type:expr,
     $encode_pat:pat => $encode_expr:expr,
     $decode_pat:pat => $decode_expr:expr
    ) => {
        impl InferType for $rust_type {
            fn infer_type() -> Type {
                $infer_type
            }
        }

        impl Encode for $rust_type {
            fn encode($encode_pat: $rust_type) -> Value {
                $encode_expr
            }
        }

        impl Decode for $rust_type {
            fn decode(val: Value) -> Option<Self> {
                match val {
                    $decode_pat => Some($decode_expr),
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }
        }
    };
}

impl_encode_decode!((), Type::Nil, () => Value::Nil, Value::Nil => ());
impl_encode_decode!(i64, Type::Int, n => Value::Int(n), Value::Int(n) => n);
impl_encode_decode!(String, Type::String, s => Value::String(s.into()), Value::String(s) => clone_or_take_arc(s));
impl_encode_decode!(Value, Type::Any, v => v, v => v);

impl InferType for &str {
    fn infer_type() -> Type {
        Type::String
    }
}

impl Encode for &str {
    fn encode(s: Self) -> Value {
        Value::String(Arc::new(s.to_string()))
    }
}

pub(crate) trait InferSignature {
    fn infer_signature() -> Signature;
}

impl<RFn> InferSignature for RFn
where
    RFn: RpcFunction,
    RFn::Domain: InferType,
    RFn::Range: InferType,
{
    fn infer_signature() -> Signature {
        Signature {
            domain: RFn::Domain::infer_type(),
            range: RFn::Range::infer_type(),
        }
    }
}

fn clone_or_take_arc<T: Clone>(arc: Arc<T>) -> T {
    match Arc::try_unwrap(arc) {
        Ok(t) => t,
        Err(arc) => (*arc).clone(),
    }
}

pub trait DecodeTypeCheck: Decode {
    fn decode_typeck(ty: &Type, val: Value) -> Result<Self, TypeMismatchError>;
}

pub trait EncodeTypeCheck: Encode {
    fn encode_typeck(ty: &Type, val: Self) -> Result<Value, TypeMismatchError>;
    fn encode_infer(val: Self) -> Value
    where
        Self: InferType + Sized;
}

impl<T: Decode> DecodeTypeCheck for T {
    fn decode_typeck(ty: &Type, val: Value) -> Result<Self, TypeMismatchError> {
        ty.check(&val)?;
        let val_ = val.clone();
        T::decode(val).ok_or_else(|| TypeMismatchError::rust_type::<T>(&val_))
    }
}

impl<T: Encode> EncodeTypeCheck for T {
    fn encode_typeck(ty: &Type, val: Self) -> Result<Value, TypeMismatchError> {
        let val = Self::encode(val);
        ty.check(&val)?;
        Ok(val)
    }

    fn encode_infer(val: Self) -> Value
    where
        Self: InferType + Sized,
    {
        Self::encode_typeck(&Self::infer_type(), val).expect("Inferred type was wrong")
    }
}
