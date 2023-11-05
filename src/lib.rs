// #![warn(unused_crate_dependencies)]
// #![allow(unused_imports, dead_code, unused_variables)]

pub mod net;
pub mod runner;
pub mod types;

use std::future::Future;
use types::{Decode, Encode, InferType, Signature};

pub trait RpcFunction {
    type Domain: Decode;
    type Range: Encode;
    type RangeFut: Future<Output = Self::Range> + Send;

    fn name(&self) -> &str;
    fn call(&self, args: Self::Domain) -> Self::RangeFut;
    fn signature(&self) -> Option<Signature> {
        None
    }
}

pub trait InferSignature {
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
