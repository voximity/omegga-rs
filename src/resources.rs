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

/// Ghost brick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostBrick {
    #[serde(rename = "targetGrid")]
    pub target_grid: String,
    pub location: (f64, f64, f64),
    pub orientation: u8,
}

/// Player paint data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPaint {
    #[serde(rename = "materialIndex")]
    pub material_index: u32,
    #[serde(rename = "materialAlpha")]
    pub material_alpha: u32,
    #[serde(rename = "material")]
    pub material: String,
    #[serde(rename = "color")]
    pub color: (u8, u8, u8),
}

/// Bounds data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateBounds {
    #[serde(rename = "minBound")]
    pub min_bound: (f64, f64, f64),
    #[serde(rename = "maxBound")]
    pub max_bound: (f64, f64, f64),
    pub center: (f64, f64, f64),
}
