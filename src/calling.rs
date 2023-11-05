use crate::{
    types::{Decode, Encode, Signature, TypeMismatch, Value},
    InferSignature, RpcFunction,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;

/// A set of [`RpcFunction`]s that can be called by name and given [`Value`]s as arguments.
#[derive(Default)]
pub struct Dispatcher {
    rpc_functions: BTreeMap<String, Arc<dyn DynamicRpcFunction + Send + Sync + 'static>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<RFn>(&mut self, rfn: RFn)
    where
        RFn: RpcFunction + Send + Sync + 'static,
        RFn::Domain: Send,
    {
        self.insert(CallableRpcFunction::new(rfn));
    }

    pub fn add_infer_signature<RFn>(&mut self, rfn: RFn)
    where
        RFn: RpcFunction + InferSignature + Send + Sync + 'static,
        RFn::Domain: Send,
    {
        self.insert(CallableRpcFunction::new_infer_signature(rfn));
    }

    fn insert<RFn>(&mut self, crfn: CallableRpcFunction<RFn>)
    where
        RFn: RpcFunction + Send + Sync + 'static,
        RFn::Domain: Send,
    {
        let name = crfn.name().to_owned();
        let dyn_rfn = Arc::new(crfn);
        self.rpc_functions.insert(name, dyn_rfn);
    }

    pub async fn call(&self, name: &str, args: Value) -> CallResult {
        Ok(self
            .rpc_functions
            .get(name)
            .ok_or(DispatchError::NoSuchFunction)?
            .call(args)
            .await?)
    }
}

pub type CallResult = Result<Value, DispatchError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcFunctionInfo {
    name: String,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum DispatchError {
    #[error("no function with given name")]
    NoSuchFunction,

    #[error("calling function: {0}")]
    CallError(#[from] CallError),
}

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum CallError {
    #[error("domain type mismatch: {0}")]
    Domain(TypeMismatch),

    #[error("range type mismatch: {0}")]
    Range(TypeMismatch),
}

struct CallableRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    rpc_function: RFn,
    signature: Signature,
}

impl<RFn> CallableRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    fn new(rpc_function: RFn) -> Self {
        let signature = rpc_function.signature().unwrap();
        Self {
            rpc_function,
            signature,
        }
    }

    fn new_infer_signature(rpc_function: RFn) -> Self
    where
        RFn: InferSignature,
    {
        Self {
            rpc_function,
            signature: RFn::infer_signature(),
        }
    }

    fn name(&self) -> &str {
        self.rpc_function.name()
    }

    async fn call(&self, args: Value) -> Result<Value, CallError> {
        let Signature { domain, range } = &self.signature;
        let decoded_args = RFn::Domain::decode(domain, args).map_err(CallError::Domain)?;
        let retval = self.rpc_function.call(decoded_args).await;
        let encoded_retval = RFn::Range::encode(range, retval).map_err(CallError::Range)?;
        Ok(encoded_retval)
    }
}

trait DynamicRpcFunction {
    fn name(&self) -> &str;
    fn call(&self, args: Value) -> BoxFuture<Result<Value, CallError>>;
}

impl<RFn> DynamicRpcFunction for CallableRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::Domain: Send,
{
    fn name(&self) -> &str {
        self.name()
    }

    fn call(&self, args: Value) -> BoxFuture<Result<Value, CallError>> {
        Box::pin(self.call(args))
    }
}