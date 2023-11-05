use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Type {
    Nil,
    Int,
    String,
    OneOf(BTreeSet<Value>),
    Any,
}

impl Type {
    fn check(&self, val: &Value) -> Result<(), TypeMismatch> {
        match (self, val) {
            // Good type checks
            (Type::Nil, Value::Nil) => (),
            (Type::Int, Value::Int(_)) => (),
            (Type::String, Value::String(_)) => (),
            (Type::OneOf(vals), val) if vals.contains(val) => (),
            (Type::Any, _) => (),

            // All else fails
            _ => Err(TypeMismatch::new(self, val))?,
        };
        Ok(())
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
                for v in vals {
                    write!(f, "{v:?}")?;
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")?;
            }
            Any => f.write_str("Any")?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatch(String);

impl TypeMismatch {
    fn new(typ: &Type, val: &Value) -> Self {
        Self(format!("Type mismatch: {val:?} :/: {typ}"))
    }
}

impl fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for TypeMismatch {}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Signature {
    pub domain: Type,
    pub range: Type,
}

#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum Value {
    Nil,
    Int(i64),
    String(String),
}

pub trait InferType {
    fn infer_type() -> Type;
}

pub trait Encode {
    fn encode(typ: &Type, val: Self) -> Result<Value, TypeMismatch>;

    fn encode_infer(val: Self) -> Value
    where
        Self: InferType + Sized,
    {
        Self::encode(&Self::infer_type(), val).expect("Inferred type was wrong")
    }
}

pub trait Decode: Sized {
    fn decode(typ: &Type, val: Value) -> Result<Self, TypeMismatch>;
}

macro_rules! impl_encode_decode {
    ($rust_type:ty, $infer_type:expr, $encode_name:pat => $encode_expr:expr, $($from_rpc_arm:tt)*) => {
        impl InferType for $rust_type {
            fn infer_type() -> Type {
                $infer_type
            }
        }

        impl Encode for $rust_type {
            fn encode(typ: &Type, $encode_name: $rust_type) -> Result<Value, TypeMismatch> {
                let val = $encode_expr;
                typ.check(&val)?;
                Ok(val)
            }
        }

        impl Decode for $rust_type {
            fn decode(typ: &Type, val: Value) -> Result<Self, TypeMismatch> {
                typ.check(&val)?;
                Ok(match val {
                    $($from_rpc_arm)*,
                    #[allow(unreachable_patterns)]
                    _ => unreachable!("Type checking is incorrect")
                })
            }
        }
    };
}

impl_encode_decode!((), Type::Nil, () => Value::Nil, Value::Nil => ());
impl_encode_decode!(i64, Type::Int, n => Value::Int(n), Value::Int(n) => n);
impl_encode_decode!(String, Type::String, s => Value::String(s), Value::String(s) => s);
impl_encode_decode!(Value, Type::Any, v => v, v => v);

impl InferType for &str {
    fn infer_type() -> Type {
        Type::String
    }
}

impl Encode for &str {
    fn encode(typ: &Type, s: Self) -> Result<Value, TypeMismatch> {
        let val = Value::String(s.to_string());
        typ.check(&val)?;
        Ok(val)
    }
}
