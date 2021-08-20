# omegga-rs

Experimental RPC interface library for Rust.

## Usage

The following is a sample plugin:

```rs
use omegga::{Omegga, rpc};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let omegga = Omegga::new();
    let mut messages = omegga.spawn();

    while let Some(message) = messages.recv().await {
        match message {
            rpc::Message::Request { method, id, .. } if method == "init" || method == "stop" => {
                // just write anything so omegga knows we work
                omegga.write_response(id, None, None);

                if method == "init" {
                    omegga.write_notification("log", Some(Value::String("Hello from omegga-rs!".into())));
                }
            }
            rpc::Message::Notification { method, params, .. } if method == "chatcmd:test" => {
                let params = params.unwrap();
                let user = params.as_array().unwrap().first().unwrap().as_str().unwrap();
                omegga.write_notification("broadcast", Some(Value::String(format!("You ran the test command, {}", user))));
            }
            _ => ()
        }
    }
}
```

It is recommended to check the [Omegga RPC reference](https://github.com/brickadia-community/omegga#json-rpc-plugins) as this library provides little abstraction.

## Credits

* voximity - creator, maintainer
* Meshiest - Omegga
