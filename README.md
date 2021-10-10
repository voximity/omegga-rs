# omegga-rs

Omegga RPC interface library for Rust.

## Usage

To enable support for serializing/deserializing into [brickadia-rs](https://github.com/brickadia-community/brickadia-rs)
save objects, use the optional feature `brs`:

```toml
omegga = { version = "0.3", features = "brs" }
```

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

*TODO: Update for 0.3*

It is recommended to check the [Omegga RPC reference](https://github.com/brickadia-community/omegga#json-rpc-plugins).

## Credits

* voximity - creator, maintainer
* Meshiest - Omegga
