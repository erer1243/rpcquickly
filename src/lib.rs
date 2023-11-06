// #![warn(unused_crate_dependencies)]
// #![allow(unused_imports, dead_code, unused_variables)]

mod calling;
mod net;
mod types;

use std::future::Future;

pub use net::{client::Client, server::Server};
pub use types::{Decode, Encode, InferType, Type, Value};

/// A single RPC function.
pub trait RpcFunction {
    /// The parameter type of the function
    type Domain: Decode;

    /// The return type of the function
    type Range: Encode;

    /// The future that represents one call of the function,
    /// i.e. one that resolves to [`Range`](Self::Range).
    type RangeFut: Future<Output = Self::Range> + Send;

    /// Provide a name to identify the function over-the-wire.
    fn name(&self) -> &str;

    /// The body of the function.
    fn call(&self, args: Self::Domain) -> Self::RangeFut;

    /// Provide a custom over-the-wire signature for the function.
    /// You *must* implement this to use [`Server::add`], or you must
    /// use [`Server::add_infer_signature`] (which requires that
    /// [`Domain`](Self::Domain) and [`Range`](Self::Range) implement [`InferType`]).
    /// Calling [`Server::add`] with an unspecified signature results in a panic.
    fn signature(&self) -> Option<(Type, Type)> {
        None
    }
}
