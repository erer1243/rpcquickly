// #![warn(unused_crate_dependencies)]
// #![allow(unused_imports, dead_code, unused_variables)]

mod calling;
mod net;
mod types;

pub use net::{client::Client, server::Server};

use std::future::Future;
pub use types::{Decode, Encode, InferType, Type, Value};

/// A single RPC function.
///
///
pub trait RpcFunction {
    type Domain: Decode;
    type Range: Encode;
    type RangeFut: Future<Output = Self::Range> + Send;

    fn name(&self) -> &str;
    fn call(&self, args: Self::Domain) -> Self::RangeFut;

    fn signature(&self) -> Option<(Type, Type)> {
        None
    }
}
