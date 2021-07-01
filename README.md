# omegga-rs

Experimental RPC interface library for Rust.

This is my first project with Rust, so don't guarantee great code or good practices! Let me know if you find better ways to do stuff.

## Usage

The following is a sample plugin:

```rs
use omegga::{Omegga, OmeggaWrapper, serde_json::{self, Value, json}, smol, rpc::RpcMessage}

fn main() {
    let mut o = OmeggaWrapper::new();

    // create a clone of the stream Arcs
    let stream_in = o.stream_receiver.clone();
    let stream_out = o.stream_sender.clone();

    // create a clone of the inner Omegga Arc
    let omegga = o.clone_inner();

    smol::spawn(async move {
        while let Ok(message) = stream_in.recv().await {
            match message {
                RpcMessage::Notification { method, params, .. } => {
                    match method.as_str() {
                        "chatcmd:test" => {
                            let user = (&params.unwrap()[0]).as_str().unwrap(); // get the runner's username
                            Omegga::notify("broadcast", json!(format!("You ran the test command, {}", user)));
                        },
                        _ => ()
                    }
                },
                RpcMessage::Request { id, method, params, .. } => {
                    match method.as_str() {
                        "init" => {
                            // send a blank response to let omegga know we work
                            stream_out.send(RpcMessage::response(id, Some(json!({})), None)).await.unwrap();

                            // print out some text
                            Omegga::notify("log", json!("Hello from omegga-rs!"));
                        },
                        "stop" => {
                            // respond with something to imply we work
                            stream_out.send(RpcMessage::response(id, Some(json!(0)), None)).await.unwrap();
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        }
    }).detach();

    o.start();
}
```

It is recommended to check the [Omegga RPC reference](https://github.com/brickadia-community/omegga#json-rpc-plugins) as this library provides little abstraction.

## Credits

* voximity - creator, maintainer
* Meshiest - Omegga
