use rpcquickly::{call, name, signature, Client, RpcFunction, Server};
use tokio::task;

pub struct Hello;

impl RpcFunction for Hello {
    name!("Hello");
    signature!(infer);
    call! {
        async fn call(&self, name: String) -> String {
            format!("Hello, {name}!")
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut server = Server::new();
    server.insert(Hello);
    task::spawn(server.serve_tcp(8888));

    // Allow server task to spin up
    task::yield_now().await;

    let mut client = Client::connect("127.0.0.1:8888").await.unwrap();
    client.ping().await.unwrap();
    let retval: String = client.call("Hello", "world").await.unwrap();
    assert_eq!(retval, "Hello, world!");
}
