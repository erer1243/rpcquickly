// #![warn(unused_crate_dependencies)]
#![allow(unused_imports, dead_code, unused_variables)]

pub mod server;
pub mod types;

use std::future::Future;
use types::{Decode, Encode};

pub trait RpcFunction<Domain, Range>
where
    Domain: Decode,
    Range: Encode,
{
    type RangeFut: Future<Output = Range>;
    fn call(&self, args: Domain) -> Self::RangeFut;
}

struct Ping;

impl RpcFunction<(), ()> for Ping {}
