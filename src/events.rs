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
    Interact(BrickInteraction),
    Event {
        name: String,
        player: Player,
        args: Vec<String>,
    },
    Autorestart(Value),
}

/// A player from interact.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlayerInteract {
    pub name: String,
    pub id: String,
    pub controller: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BrickInteraction {
    pub brick_asset: String,
    pub brick_name: String,
    pub brick_size: Vec<u64>,
    pub player: PlayerInteract,
    pub position: (f64, f64, f64),
    pub data: Option<Value>,
    pub error: bool,
    pub json: bool,
    pub message: String
}
