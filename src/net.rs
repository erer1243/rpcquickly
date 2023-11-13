pub mod client;
pub mod server;

use crate::{
    dispatcher::{DispatchError, RpcFunctionInfo},
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
    Call(Result<Value, DispatchError>),
}
