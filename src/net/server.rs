use crate::{
    dispatcher::Dispatcher,
    net::{Request, Response},
    RpcFunction,
};
use async_bincode::tokio::AsyncBincodeStream;
use delegate::delegate;
use futures::{SinkExt, StreamExt};
use std::{io, net::Ipv4Addr, sync::Arc};
use tokio::{io::BufStream, net::TcpListener, task};

#[derive(Default)]
pub struct Server {
    dispatcher: Dispatcher,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    delegate! {
        to self.dispatcher {
            pub fn add<RFn>(&mut self, rfn: RFn)
            where
                RFn: RpcFunction + Send + Sync + 'static,
                RFn::Domain: Send;
        }
    }

    async fn handle_request(self: &Arc<Self>, req: Request) -> Response {
        match req {
            Request::Ping => Response::Ping,
            Request::Call { name, args } => Response::Call(self.dispatcher.call(&name, args).await),
            Request::RpcFunctions => Response::RpcFunctions(self.dispatcher.rpc_functions()),
            // _ => Response::Other("unimplemented".to_owned()),
        }
    }

    pub async fn serve_tcp(self, port: u16) -> io::Result<()> {
        let root_arc = Arc::new(self);
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
        loop {
            let arc_self = root_arc.clone();
            let (sock, _addr) = listener.accept().await?;
            let mut sock =
                AsyncBincodeStream::<_, Request, Response, _>::from(BufStream::new(sock))
                    .for_async();

            task::spawn(async move {
                while let Some(Ok(request)) = sock.next().await {
                    let response = arc_self.handle_request(request).await;
                    _ = sock.send(response).await;
                }
            });
        }
    }
}
