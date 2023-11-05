use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Type {
    Nil,
    String,
    Int,
}

impl Type {
    fn name(&self) -> &'static str {
        use Type::*;
        match self {
            Nil => "Nil",
            String => "String",
            Int => "Int",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())?;
        // if let Type::Enum { values } = self {
        //     f.write_str("(")?;
        //     #[allow(unstable_name_collisions)]
        //     values
        //         .iter()
        //         .map(|s| s.as_str())
        //         .intersperse(", ")
        //         .map(|s| f.write_str(s))
        //         .collect::<fmt::Result>()?;
        //     f.write_str(")")?;
        // }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Value {
    Nil,
    String(String),
    Int(i64),
}

pub trait Typed {
    fn rpc_type() -> Type;
}

pub trait Encode {
    fn encode(val: Self) -> Value;
}

pub trait Decode: Sized {
    fn decode(typ: Type, val: Value) -> Result<Self, TypeMismatch>;
}

macro_rules! impl_encode_decode {
    ($rust_type:ty, $rpc_type:expr, $encode_name:pat => $encode_expr:expr, $($from_rpc_arm:tt)*) => {
        impl Typed for $rust_type {
            fn rpc_type() -> Type {
                $rpc_type
            }
        }

        impl Encode for $rust_type {
            fn encode($encode_name: $rust_type) -> Value {
                $encode_expr
            }
        }

        impl Decode for $rust_type {
            fn decode(_typ: Type, val: Value) -> Result<Self, TypeMismatch> {
                Ok(match val {
                    $($from_rpc_arm)*,
                    _ => return Err(TypeMismatch::new(val, <Self as Typed>::rpc_type()))
                })
            }
        }
    };
}

impl_encode_decode!((), Type::Nil, () => Value::Nil, Value::Nil => ());
impl_encode_decode!(String, Type::String, s => Value::String(s), Value::String(s) => s);
impl_encode_decode!(i64, Type::Int, n => Value::Int(n), Value::Int(n) => n);

#[derive(Debug, Clone)]
pub struct TypeMismatch {
    value: Value,
    expected_type: Type,
}

impl TypeMismatch {
    fn new(value: Value, expected_type: Type) -> Self {
        Self {
            value,
            expected_type,
        }
    }
}

impl fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type error: {:?} :/: {}", self.value, self.expected_type)
    }
}

impl Error for TypeMismatch {}
