use super::{Request, Response};
use crate::{
    calling::RpcFunctionInfo,
    types::{Decode, DecodeTypeCheck, Encode, InferType},
};
use async_bincode::{tokio::AsyncBincodeStream, AsyncDestination};
use futures::{SinkExt, StreamExt};
use std::io;
use tokio::{
    io::BufStream,
    net::{TcpStream, ToSocketAddrs},
};

pub struct Client(AsyncBincodeStream<BufStream<TcpStream>, Response, Request, AsyncDestination>);

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Client> {
        let sock = TcpStream::connect(addr).await?;
        let sock = BufStream::new(sock);
        let sock = AsyncBincodeStream::from(sock).for_async();
        Ok(Client(sock))
    }

    async fn send_recv(&mut self, req: Request) -> Result<Response, String> {
        self.0.send(req).await.map_err(|e| e.to_string())?;
        let resp = self
            .0
            .next()
            .await
            .ok_or_else(|| "No response from server".to_string())?
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }

    pub async fn ping(&mut self) -> Result<(), String> {
        let resp = self.send_recv(Request::Ping).await?;
        match resp {
            Response::Ping => Ok(()),
            other => Err(format!("Unexpected response: {other:?}")),
        }
    }

    pub async fn rpc_functions(&mut self) -> Result<Vec<RpcFunctionInfo>, String> {
        let resp = self.send_recv(Request::RpcFunctions).await?;
        match resp {
            Response::RpcFunctions(funcs) => Ok(funcs),
            other => Err(format!("Unexpected response: {other:?}")),
        }
    }

    pub async fn call<Domain, Range>(&mut self, name: &str, args: Domain) -> Result<Range, String>
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
