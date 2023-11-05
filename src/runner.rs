use std::{collections::BTreeMap, sync::Arc};

use crate::{
    types::{Decode, Encode, Signature, TypeMismatch, Value},
    InferSignature, RpcFunction,
};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

/// A set of [`RpcFunction`]s that can be called by name and given [`Value`]s as arguments.
#[derive(Default)]
pub struct RpcFunctionRunner {
    rpc_functions: BTreeMap<String, Arc<dyn ObjectSafeRpcFunction>>,
}

impl RpcFunctionRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_func_infer_signature<RFn>(&mut self, rfn: RFn)
    where
        RFn: RpcFunction + InferSignature + Send + Sync + 'static,
    {
        let sig = RFn::infer_signature();
        self.push_func_explicit_signature(rfn, sig);
    }

    pub fn push_func_explicit_signature<RFn>(&mut self, rpc_function: RFn, signature: Signature)
    where
        RFn: RpcFunction + Send + Sync + 'static,
    {
        let name = rpc_function.name().to_owned();
        let typed_rpc_function = Arc::new(TypedRpcFunction {
            rpc_function,
            signature,
        });
        self.rpc_functions.insert(name, typed_rpc_function);
    }

    pub async fn call(&self, name: &str, args: Value) -> CallResult {
        use CallError::*;
        let rfn = self.rpc_functions.get(name).ok_or(NoSuchFunction)?;
        rfn.call(args)
            .map_err(DomainType)? // Domain type check
            .await
            .map_err(RangeType) // Range type check
    }
}

pub type CallResult = Result<Value, CallError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcFunctionInfo {
    name: String,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CallError {
    NoSuchFunction,
    DomainType(TypeMismatch),
    RangeType(TypeMismatch),
}

struct TypedRpcFunction<RFn> {
    rpc_function: RFn,
    signature: Signature,
}

type TypeCheckResult<T> = Result<T, TypeMismatch>;

trait ObjectSafeRpcFunction: Send + Sync {
    fn name(&self) -> &str;
    fn call(&self, args: Value) -> TypeCheckResult<BoxFuture<TypeCheckResult<Value>>>;
}

impl<RFn> ObjectSafeRpcFunction for TypedRpcFunction<RFn>
where
    RFn: RpcFunction + Send + Sync,
    RFn::RangeFut: Send,
{
    fn name(&self) -> &str {
        self.rpc_function.name()
    }

    fn call(&self, args: Value) -> TypeCheckResult<BoxFuture<TypeCheckResult<Value>>> {
        // Type check & decode the given arguments
        let decoded_args = RFn::Domain::decode(&self.signature.domain, args)?;
        let call_fut = self.rpc_function.call(decoded_args);
        let call_fut = async move {
            // Type check & encode the returned value
            let retval = call_fut.await;
            let encoded = RFn::Range::encode(&self.signature.range, retval)?;
            Ok(encoded)
        };
        Ok(Box::pin(call_fut))
    }
}
