pub mod client;
pub mod server;

use crate::{
    dispatcher::{CallResult, RpcFunctionInfo},
    types::Value,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Request {
    Ping,
    RpcFunctions,
    Call { name: String, args: Value },
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Response {
    Ping,
    RpcFunctions(Vec<RpcFunctionInfo>),
    Call(CallResult),
    Other(String),
}
