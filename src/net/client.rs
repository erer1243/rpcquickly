use super::{Request, Response};
use crate::types::{Decode, DecodeTypeCheck, Encode, InferType};
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

    pub async fn ping(&self) -> Result<(), String> {
        let resp = self.send_recv(Request::Ping).await?;
        match resp {
            Response::Ping => Ok(()),
            other => Err(format!("Unexpected response: {other:?}")),
        }
    }

    pub async fn call<Domain, Range>(&self, name: &str, args: Domain) -> Result<Range, String>
    where
        Domain: Encode + InferType,
        Range: Decode + InferType,
    {
        let req = Request::Call {
            name: name.to_string(),
            args: Domain::encode(args),
        };
        let resp = self.send_recv(req).await?;
        match resp {
            Response::Call(res) => match res {
                Ok(val) => {
                    Range::decode_typeck(&Range::infer_type(), val).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            },
            other => Err(format!("Unexpected response: {other:?}")),
        }
    }
}
