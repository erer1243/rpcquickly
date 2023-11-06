use futures::future::{ready, Ready};
use rpcquickly::{Client, RpcFunction, Server};
use std::time::Duration;
use tokio::task;

pub struct Hello;

impl RpcFunction for Hello {
    type Domain = String;
    type Range = String;
    type RangeFut = Ready<String>;

    fn name(&self) -> &str {
        "Hello"
    }

    fn call(&self, name: String) -> Self::RangeFut {
        ready(format!("Hello, {name}!"))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut server = Server::new();
    server.add_infer_signature(Hello);
    task::spawn(server.serve_tcp(8888));
    tokio::time::sleep(Duration::from_secs_f32(0.01)).await;

    let mut client = Client::connect("127.0.0.1:8888").await.unwrap();
    client.ping().await.unwrap();
    let retval: String = client.call("Hello", "world").await.unwrap();
    println!("{retval}");
}
