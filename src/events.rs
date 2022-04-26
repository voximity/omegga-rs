use serde_json::Value;

use crate::{resources::Player, rpc::RequestId};

#[derive(Debug, Clone)]
pub enum Event {
    Init {
        id: RequestId,
        config: Value,
    },
    Stop {
        id: RequestId,
    },
    Bootstrap {
        omegga: Value,
    },
    PluginPlayersRaw {
        players: Vec<Player>,
    },
    PluginEmit {
        id: RequestId,
        event: String,
        from: String,
        args: Vec<Value>,
    },
    Line(String),
    Start {
        map: String,
    },
    Host {
        name: String,
        id: String,
    },
    Version(Value),
    Unauthorized,
    Join(Player),
    Leave(Player),
    Command {
        player: String,
        command: String,
        args: Vec<String>,
    },
    ChatCommand {
        player: String,
        command: String,
        args: Vec<String>,
    },
    Chat {
        player: String,
        message: String,
    },
    MapChange(String),
    Interact {
        brick_asset: String,
        player: Player,
        position: (f64, f64, f64),
        data: Option<Value>,
    },
    Event {
        name: String,
        player: Player,
        args: Vec<String>,
    },
    Autorestart(Value),
}
