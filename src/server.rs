use crate::{
    types::{Decode, Encode},
    RpcFunction,
};
use futures::{
    future::{BoxFuture, Map},
    stream::{FuturesUnordered, Next, NextIfEq},
    FutureExt, Stream, StreamExt,
};
use pin_project_lite::pin_project;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    pub struct RpcFunctionCallManager<'a, RFn, CallerId>
    where
        RFn: RpcFunction,
    {
        rpc_function: RFn,

        #[pin]
        calls: FuturesUnordered<BoxFuture<'a, (CallerId, RFn::Range)>>,
    }
}

impl<'a, RFn, CallerId> RpcFunctionCallManager<'a, RFn, CallerId>
where
    RFn: RpcFunction,
    RFn::RangeFut: Send + 'a,
    CallerId: Send + 'a,
{
    pub fn new(rpc_function: RFn) -> Self {
        Self {
            rpc_function,
            calls: FuturesUnordered::new(),
        }
    }

    pub fn call(&self, caller_id: CallerId, args: RFn::Domain) {
        let fut = self
            .rpc_function
            .call(args)
            .map(move |retval| (caller_id, retval));
        self.calls.push(Box::pin(fut));
    }
}

impl<'a, RFn: RpcFunction, CallerId> Stream for RpcFunctionCallManager<'a, RFn, CallerId> {
    type Item = (CallerId, RFn::Range);

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().calls.poll_next(cx)
    }
}
