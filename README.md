# omegga-rs

Omegga RPC interface library for Rust.

## Usage

To enable support for serializing/deserializing into [brickadia-rs](https://github.com/brickadia-community/brickadia-rs)
save objects, use the optional feature `brs`:

```toml
omegga = { version = "1.0", features = "brs" }
```

The following is a sample plugin:

```rs
use omegga::{events::Event, Omegga};

#[tokio::main]
async fn main() {
    let omegga = Omegga::new();
    let mut events = omegga.spawn();

    while let Some(event) = events.recv().await {
        match event {
            // Register our commands on init...
            Event::Init { id, .. } => omegga.register_commands(id, &["ping"]),

            // Send a blank response when we're told to stop...
            Event::Stop { id, .. } => omegga.write_response(id, None, None),

            // Listen to commands sent to the plugin...
            Event::Command {
                player, command, ..
            } => match command.as_str() {
                // When the command matches `ping`, send `Pong!`
                "ping" => omegga.whisper(player, "Pong!"),
                _ => (),
            },

            _ => (),
        }
    }
}
```

It is recommended to check the [Omegga RPC reference](https://github.com/brickadia-community/omegga#json-rpc-plugins).

## Credits

* voximity - creator, maintainer
* Meshiest - Omegga
