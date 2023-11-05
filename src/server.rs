use crate::{
    types::{Decode, Encode, Signature, TypeMismatch, Value},
    InferSignature, RpcFunction,
};
use futures::{future::BoxFuture, FutureExt};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Server {
    rpc_functions: BTreeMap<String, Box<dyn ObjectSafeRpcFunction>>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
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

type TypeCheckedValueFuture<'a> = Result<BoxFuture<'a, Result<Value, TypeMismatch>>, TypeMismatch>;

trait ObjectSafeRpcFunction {
    fn name(&self) -> &str;
    fn call(&self, args: Value) -> TypeCheckedValueFuture;
}

struct TypedRpcFunction<RFn> {
    rpc_function: RFn,
    signature: Signature,
}

impl<RFn> TypedRpcFunction<RFn>
where
    RFn: RpcFunction,
{
    fn new(rpc_function: RFn, signature: Signature) -> Self {
        Self {
            rpc_function,
            signature,
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

    fn call(&self, args: Value) -> TypeCheckedValueFuture {
        let decoded_args = RFn::Domain::decode(&self.signature.domain, args)?;
        let range = self.signature.range.clone();
        let fut = self
            .rpc_function
            .call(decoded_args)
            .map(move |retval| RFn::Range::encode(&range, retval));
        Ok(Box::pin(fut))
    }
}
