// #![warn(unused_crate_dependencies)]
// #![allow(unused_imports, dead_code, unused_variables)]

mod dispatcher;
mod macros;
mod net;
mod types;

pub use futures::future::BoxFuture;

pub use dispatcher::RpcFunctionInfo;
pub use net::{client::Client, server::Server};
pub use types::{Decode, Encode, InferType, Signature, Type, Value};

/// A single RPC function.
pub trait RpcFunction {
    /// The parameter type of the function
    type Domain: Decode;

    /// The return type of the function
    type Range: Encode;

    /// Provide a name to identify the function over-the-wire.
    fn name(&self) -> &str;

    /// The body of the function.
    fn call<'a>(&'a self, args: Self::Domain) -> BoxFuture<'a, Self::Range>;

    /// Provide the over-the-wire signature for the function.
    fn signature(&self) -> Signature;
}
