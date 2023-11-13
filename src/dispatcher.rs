use crate::{
    types::{DecodeTypeCheck, EncodeTypeCheck, Signature, TypeMismatchError, Value},
    RpcFunction,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;

/// The name and signature of an rpc function
#[derive(Serialize, Deserialize, Debug)]
pub struct RpcFunctionInfo {
    pub name: String,
    pub signature: Signature,
}

/// A map from name to a dynamically-type-checked rpc function.
/// Held rpc functions can be called or queried
#[derive(Default)]
pub(crate) struct Dispatcher {
    rpc_functions: BTreeMap<String, Arc<dyn DynamicRpcFunction + Send + Sync + 'static>>,
}

impl Dispatcher {
    pub(crate) fn insert<RFn>(&mut self, rpc_function: RFn)
    where
        RFn: RpcFunction + Send + Sync + 'static,
        RFn::Domain: Send,
    {
        let name = rpc_function.name().to_owned();
        let signature = rpc_function.signature();
        let rfws = Arc::new(UntypedRpcFunction {
            signature,
            rpc_function,
        });
        self.rpc_functions.insert(name, rfws);
    }

    pub(crate) async fn call(&self, name: &str, args: Value) -> Result<Value, DispatchError> {
        Ok(self
            .rpc_functions
            .get(name)
            .ok_or(DispatchError::NoSuchFunction)?
            .call_typecked(args)
            .await?)
    }

    pub(crate) fn rpc_functions(&self) -> Vec<RpcFunctionInfo> {
        self.rpc_functions
            .iter()
            .map(|(name, rfn)| RpcFunctionInfo {
                name: name.clone(),
                signature: rfn.signature().clone(),
            })
            .collect()
    }
}

/// A type-erased version of the main trait, RpcFunction.
/// Object-safe, so it can be stored as a trait object along with other
/// `RpcFunction`s in `Dispatcher`.
trait DynamicRpcFunction {
    fn signature(&self) -> &Signature;
    fn call_typecked(&self, args: Value) -> BoxFuture<Result<Value, CallError>>;
}

/// A struct to implement DynamicRpcFunction
struct UntypedRpcFunction<RFn> {
    signature: Signature,
    rpc_function: RFn,
}

impl<RFn> DynamicRpcFunction for UntypedRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn call_typecked(&self, args: Value) -> BoxFuture<Result<Value, CallError>> {
        let body = async move {
            let Signature { domain, range } = &self.signature;
            let decoded_args =
                RFn::Domain::decode_typecked(domain, args).map_err(CallError::Domain)?;
            let retval = self.rpc_function.call(decoded_args).await;
            let encoded_retval =
                RFn::Range::encode_typecked(range, retval).map_err(CallError::Range)?;
            Ok(encoded_retval)
        };
        Box::pin(body)
    }
}

#[derive(Serialize, Deserialize, Debug, Error)]
pub(crate) enum DispatchError {
    #[error("no function with given name")]
    NoSuchFunction,

    #[error("calling function: {0}")]
    CallError(#[from] CallError),
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub(crate) enum CallError {
    #[error("domain type mismatch: {0}")]
    Domain(TypeMismatchError),

    #[error("(BUG in RPC function) range type mismatch: {0}")]
    Range(TypeMismatchError),
}
