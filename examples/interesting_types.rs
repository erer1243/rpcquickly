use std::{net::SocketAddr, str::FromStr, time::Duration};

use futures::future::BoxFuture;
use rpcquickly::{Client, RpcFunction, Server, Type, Value};
use tokio::task;

pub struct MultipleChoice(&'static str);

impl MultipleChoice {
    fn new() -> Self {
        let ans = ["a", "b", "c", "d"][rand::random::<usize>() % 4];
        println!("The correct answer will be {ans}");
        Self(ans)
    }
}

impl RpcFunction for MultipleChoice {
    type Domain = String;
    type Range = String;
    type RangeFut = BoxFuture<'static, Self::Range>;

    fn name(&self) -> &str {
        "MultipleChoice"
    }

    fn call(&self, guess: String) -> Self::RangeFut {
        let answer = self.0;
        Box::pin(async move {
            if guess == answer {
                "right".into()
            } else {
                "wrong".into()
            }
        })
    }

    fn signature(&self) -> Option<(Type, Type)> {
        Some((
            Type::one_of(["a", "b", "c", "d"]),
            Type::one_of(["right", "wrong"]),
        ))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut server = Server::new();
    server.add(MultipleChoice::new());
    task::spawn(server.serve_tcp(8888));
    tokio::time::sleep(Duration::from_secs_f32(0.01)).await;

    let addr = SocketAddr::from_str("127.0.0.1:8888").unwrap();
    let client = Client(addr);
    client.ping().await.unwrap();
    for ans in ["a", "b", "c", "d"] {
        let retval: String = client.call("MultipleChoice", ans).await.unwrap();
        println!("ans = {retval}");
    }

    for bad_guess in [Value::from("x"), 10.into(), ().into()] {
        let err = client
            .call::<_, Value>("MultipleChoice", bad_guess)
            .await
            .unwrap_err();
        println!("{err}");
    }
}
