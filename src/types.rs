use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, error::Error, fmt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Type {
    Nil,
    String,
    Int,
    OneOf(BTreeSet<Value>),
}

impl Type {
    fn check(&self, val: &Value) -> Result<(), TypeMismatch> {
        Ok(match (self, val) {
            // Good type checks
            (Type::Nil, Value::Nil) => (),
            (Type::String, Value::String(_)) => (),
            (Type::Int, Value::Int(_)) => (),
            (Type::OneOf(vals), val) if vals.contains(val) => (),

            // All else fails
            _ => Err(TypeMismatch::new(self, val))?,
        })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self {
            Nil => f.write_str("Nil")?,
            String => f.write_str("String")?,
            Int => f.write_str("Int")?,
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
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
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
    String(String),
    Int(i64),
}

pub trait InferType {
    fn infer_type() -> Type;
}

pub trait Encode {
    fn encode(val: Self) -> Value;
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
            fn encode($encode_name: $rust_type) -> Value {
                $encode_expr
            }
        }

        impl Decode for $rust_type {
            fn decode(typ: &Type, val: Value) -> Result<Self, TypeMismatch> {
                typ.check(&val)?;
                Ok(match val {
                    $($from_rpc_arm)*,
                    _ => unreachable!("Type checking is incorrect")
                })
            }
        }
    };
}

impl_encode_decode!((), Type::Nil, () => Value::Nil, Value::Nil => ());
impl_encode_decode!(String, Type::String, s => Value::String(s), Value::String(s) => s);
impl_encode_decode!(i64, Type::Int, n => Value::Int(n), Value::Int(n) => n);
