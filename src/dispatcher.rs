use crate::{
    types::{DecodeTypeCheck, EncodeTypeCheck, Signature, TypeMismatchError, Value},
    RpcFunction,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;

/// Maps [`RpcFunction`] names to a [`CallableRpcFunction`].
///
/// Functions can be registered with [`add`] or [`add_infer_signature`].
/// Functions can be called via [`call`].
/// A list of functions can be retrieved with [`rpc_functions`].
#[derive(Default)]
pub(crate) struct Dispatcher {
    rpc_functions: BTreeMap<String, Arc<dyn DynamicRpcFunction + Send + Sync + 'static>>,
}

impl Dispatcher {
    pub(crate) fn add<RFn>(&mut self, rpc_function: RFn)
    where
        RFn: RpcFunction + Send + Sync + 'static,
        RFn::Domain: Send,
    {
        // let signature = match rpc_function.signature() {
        //     Some((domain, range)) => Signature { domain, range },
        //     None => infer_signature_or_panic::<RFn>(),
        // };
        let signature = rpc_function.signature();
        let rfws = TypedRpcFunction {
            rpc_function,
            signature,
        };
        let name = rfws.name().to_owned();
        let dyn_rfn = Arc::new(rfws);
        self.rpc_functions.insert(name, dyn_rfn);
    }

    pub(crate) async fn call(&self, name: &str, args: Value) -> CallResult {
        Ok(self
            .rpc_functions
            .get(name)
            .ok_or(DispatchError::NoSuchFunction)?
            .call(args)
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

pub(crate) type CallResult = Result<Value, DispatchError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcFunctionInfo {
    name: String,
    signature: Signature,
}

struct TypedRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    rpc_function: RFn,
    signature: Signature,
}

impl<RFn> TypedRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    async fn call(&self, args: Value) -> Result<Value, CallError> {
        let Signature { domain, range } = &self.signature;
        let decoded_args = RFn::Domain::decode_typeck(domain, args).map_err(CallError::Domain)?;
        let retval = self.rpc_function.call(decoded_args).await;
        let encoded_retval = RFn::Range::encode_typeck(range, retval).map_err(CallError::Range)?;
        Ok(encoded_retval)
    }
}

/// A type-erased version of the main trait, RpcFunction
trait DynamicRpcFunction {
    fn name(&self) -> &str;
    fn signature(&self) -> &Signature;
    fn call(&self, args: Value) -> BoxFuture<Result<Value, CallError>>;
}

impl<RFn> DynamicRpcFunction for TypedRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    fn name(&self) -> &str {
        self.rpc_function.name()
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn call(&self, args: Value) -> BoxFuture<Result<Value, CallError>> {
        Box::pin(self.call(args))
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
