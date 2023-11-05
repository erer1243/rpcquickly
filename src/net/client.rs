use super::{Request, Response};
use crate::types::{Encode, InferType};
use async_bincode::{tokio::AsyncBincodeStream, AsyncDestination};
use futures::{SinkExt, StreamExt};
use std::{io, net::SocketAddr};
use tokio::{io::BufStream, net::TcpStream};

pub struct Client(pub SocketAddr);

impl Client {
    async fn connect(
        &self,
    ) -> io::Result<AsyncBincodeStream<BufStream<TcpStream>, Response, Request, AsyncDestination>>
    {
        let sock = TcpStream::connect(self.0).await?;
        let sock = BufStream::new(sock);
        let sock = AsyncBincodeStream::from(sock).for_async();
        Ok(sock)
    }

    async fn send_recv(&self, req: Request) -> Result<Response, String> {
        let mut sock = self.connect().await.map_err(|e| e.to_string())?;
        sock.send(req).await.map_err(|e| e.to_string())?;
        let resp = sock
            .next()
            .await
            .ok_or_else(|| "No response from server".to_string())?
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }

    pub async fn ping(&self) {
        println!("{:?}", self.send_recv(Request::Ping).await);
    }

    pub async fn call<Domain: Encode + InferType>(&self, name: &str, args: Domain) {
        let req = Request::Call {
            name: name.to_string(),
            args: Domain::encode_infer(args),
        };
        println!("{:?}", self.send_recv(req).await);
    }
}
