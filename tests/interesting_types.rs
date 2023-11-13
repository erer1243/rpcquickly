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
#[test]
async fn main() {
    let mut server = Server::new();
    server.insert(MultipleChoice::new());
    task::spawn(server.serve_tcp(8888));

    task::yield_now().await;

    let mut client = Client::connect("127.0.0.1:8888").await.unwrap();
    client.ping().await.unwrap();

    let mut rights = 0;
    let mut wrongs = 0;

    for ans in ["a", "b", "c", "d"] {
        let retval: String = client.call("MultipleChoice", ans).await.unwrap();
        println!("{ans} is {retval}");

        if retval == "right" {
            rights += 1;
        } else {
            wrongs += 1;
        }
    }

    assert_eq!(rights, 1);
    assert_eq!(wrongs, 3);

    for bad_guess in [Value::from("x"), Value::from(10), Value::from(())] {
        let err = client
            .call::<_, Value>("MultipleChoice", bad_guess)
            .await
            .unwrap_err();
        println!("{err}");
    }
}
