use crate::{
    calling::Dispatcher,
    net::{Request, Response},
};
use async_bincode::tokio::AsyncBincodeStream;
use futures::{SinkExt, StreamExt};
use std::{io, net::Ipv4Addr, sync::Arc};
use tokio::{io::BufStream, net::TcpListener, task};

#[derive(Default)]
pub struct Server {
    runner: Dispatcher,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    async fn handle_request(self: Arc<Self>, req: Request) -> Response {
        match req {
            Request::Ping => Response::Ping,
            Request::Call { name, args } => Response::Call(self.runner.call(&name, args).await),
            _ => Response::Other("unimplemented".to_owned()),
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
                if let Some(Ok(request)) = sock.next().await {
                    let response = arc_self.handle_request(request).await;
                    _ = sock.send(response).await;
                }
            });
        }
    }
}
