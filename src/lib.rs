// #![warn(unused_crate_dependencies)]
#![allow(unused_imports, dead_code, unused_variables)]

pub mod server;
pub mod types;

use futures::future::{ready, Ready};
use std::future::Future;
use types::{Decode, Encode};

pub trait RpcFunction {
    type Domain: Decode;
    type Range: Encode;
    type RangeFut: Future<Output = Self::Range>;

    fn call(&self, args: Self::Domain) -> Self::RangeFut;
}

pub struct Ping;

impl RpcFunction for Ping {
    type Domain = ();
    type Range = ();
    type RangeFut = Ready<()>;

    fn call(&self, _args: ()) -> Self::RangeFut {
        ready(())
    }
}
