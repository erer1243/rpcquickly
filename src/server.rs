use crate::{
    types::{Decode, Encode, Signature, Type, TypeMismatch, Value},
    InferSignature, RpcFunction,
};
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, Stream};
use std::{
    collections::BTreeMap,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Server {
    rpc_functions: BTreeMap<String, Box<dyn ObjectSafeRpcFunction>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            rpc_functions: BTreeMap::new(),
        }
    }

    pub fn push_func_infer_signature<RFn>(&mut self, rfn: RFn)
    where
        RFn: RpcFunction + InferSignature + 'static,
    {
        let sig = RFn::infer_signature();
        self.push_func_explicit_signature(rfn, sig);
    }

    pub fn push_func_explicit_signature<RFn>(&mut self, rfn: RFn, signature: Signature)
    where
        RFn: RpcFunction + 'static,
    {
        let name = rfn.name().to_owned();
        let typed_rpc_function = Box::new(TypedRpcFunction::new(rfn, signature));
        self.rpc_functions.insert(name, typed_rpc_function);
    }
}

trait ObjectSafeRpcFunction {
    fn name(&self) -> &str;
    fn call(&self, args: Value) -> Result<BoxFuture<Value>, TypeMismatch>;
}

struct TypedRpcFunction<RFn> {
    rpc_function: RFn,
    domain: Type,
    range: Type,
}

impl<RFn> TypedRpcFunction<RFn>
where
    RFn: RpcFunction,
{
    fn new(rpc_function: RFn, Signature { domain, range }: Signature) -> Self {
        Self {
            rpc_function,
            domain,
            range,
        }
    }
}

impl<RFn> ObjectSafeRpcFunction for TypedRpcFunction<RFn>
where
    RFn: RpcFunction,
    RFn::RangeFut: Send,
{
    fn name(&self) -> &str {
        self.rpc_function.name()
    }

    fn call(&self, args: Value) -> Result<BoxFuture<Value>, TypeMismatch> {
        let decoded_args = RFn::Domain::decode(&self.domain, args)?;
        let fut = self.rpc_function.call(decoded_args).map(RFn::Range::encode);
        Ok(Box::pin(fut))
    }
}
