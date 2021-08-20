use serde::{Deserialize, Serialize};

/// A player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: String,
    pub controller: String,
    pub state: String,
}

/// A player position, which composes a `Player` and their position (a `(f64, f64, f64)`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPosition {
    pub player: Player,
    pub pos: Option<(f64, f64, f64)>,
}
