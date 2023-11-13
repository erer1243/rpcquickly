use rpcquickly::{call, name, signature, Client, RpcFunction, Server, Type, Value};
use tokio::task;

pub struct MultipleChoice {
    answer: &'static str,
}

impl MultipleChoice {
    fn new() -> Self {
        let answer = ["a", "b", "c", "d"][rand::random::<usize>() % 4];
        println!("The correct answer will be {answer}");
        Self { answer }
    }
}

impl RpcFunction for MultipleChoice {
    name!("MultipleChoice");
    signature!(Type::one_of(["a", "b", "c", "d"]) => Type::one_of(["right", "wrong"]));
    call! {
        async fn call(&self, guess: String) -> String {
            if guess == self.answer {
                "right".into()
            } else {
                "wrong".into()
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut server = Server::new();
    server.add(MultipleChoice::new());
    task::spawn(server.serve_tcp(8888));

    task::yield_now().await;

    let mut client = Client::connect("127.0.0.1:8888").await.unwrap();
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
