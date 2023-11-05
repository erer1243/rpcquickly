use crate::RpcFunction;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct RpcFunctionCallManager<'a, RFn, CallerId>
where
    RFn: RpcFunction,
{
    rpc_function: RFn,
    calls: FuturesUnordered<BoxFuture<'a, (CallerId, RFn::Range)>>,
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
        let calls = unsafe { Pin::map_unchecked_mut(self, |mgr| &mut mgr.calls) };
        calls.poll_next(cx)
    }
}
