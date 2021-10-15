use std::collections::HashMap;

use serde::{Deserialize, Serialize};

macro_rules! color_def {
    ($n:ident, $c:literal) => {
        fn $n(self) -> Self {
            self.color($c)
        }
    }
}

pub trait Colorize: Sized {
    fn bold(self) -> Self;
    fn italics(self) -> Self;
    fn color(self, code: &str) -> Self;
    fn hyperlink(self, link: &str) -> Self;
    fn size(self, size: i32) -> Self;

    color_def!(red, "f00");
    color_def!(green, "0f0");
    color_def!(blue, "00f");
    color_def!(yellow, "ff0");
    color_def!(teal, "0ff");
    color_def!(magenta, "f0f");
    color_def!(white, "fff");
    color_def!(black, "000");
    color_def!(dark_gray, "666");
    color_def!(light_gray, "bbb");
}

impl Colorize for String {
    fn bold(self) -> Self {
        format!("**{}**", self)
    }

    fn italics(self) -> Self {
        format!("*{}*", self)
    }

    fn color(self, code: &str) -> Self {
        format!("<color=\"{}\">{}</>", code, self)
    }

    fn hyperlink(self, link: &str) -> Self {
        format!("[{}]({})", self, link)
    }

    fn size(self, size: i32) -> Self {
        format!("<size=\"{}\">{}</>", self, size)
    }
}

/// A player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: String,
    pub controller: String,
    pub state: String,
    pub host: Option<bool>,
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
    pub orientation: String,
}

/// Player paint data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPaint {
    #[serde(rename = "materialIndex")]
    pub material_index: String,
    #[serde(rename = "materialAlpha")]
    pub material_alpha: String,
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

/// A plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub author: String,
    pub config: HashMap<String, ConfigEntry>,
    pub commands: Vec<Command>,
}

/// A config entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub description: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    #[serde(default)]
    pub default: serde_json::Value,
}

/// A config command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub example: String,
    pub args: Vec<CommandArg>,
}

/// A config command arg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
}
