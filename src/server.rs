use crate::{
    types::{Decode, Encode},
    RpcFunction,
};
use futures::{future::Map, stream::FuturesUnordered, FutureExt};
use std::{collections::HashMap, future::Future};

struct FunctionCallManager<RFn, Range, CallerId> {
    rpc_function: RFn,
    calls: FuturesUnordered<Box<dyn Future<Output = (CallerId, Range)>>>,
}

impl<RFn, Range, CallerId> FunctionCallManager<RFn, Range, CallerId>
where
    Range: Encode,
    CallerId: 'static,
{
    fn new(rpc_function: RFn) -> Self {
        Self {
            rpc_function,
            calls: FuturesUnordered::new(),
        }
    }

    fn call<Domain>(&self, caller_id: CallerId, args: Domain)
    where
        RFn: RpcFunction<Domain, Range>,
        RFn::RangeFut: 'static,
        Domain: Decode,
    {
        let call_fut = self
            .rpc_function
            .call(args)
            .map(move |retval| (caller_id, retval));
        self.calls.push(Box::new(call_fut));
    }
}
