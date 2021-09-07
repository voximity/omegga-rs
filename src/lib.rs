use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

#[cfg(feature = "brs")]
use brickadia::save;

use dashmap::{mapref::entry::Entry, DashMap};
use resources::{GhostBrick, Player, PlayerPaint, Plugin, TemplateBounds};
use serde_json::{json, Value};
use thiserror::Error;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::{
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
};

use crate::resources::PlayerPosition;

pub mod resources;
pub mod rpc;

pub type RpcEventReceiver = UnboundedReceiver<rpc::Message>;

/// A future that waits for the server to respond, returning a [`Response`](crate::Response).
/// This will await indefinitely, so use with Tokio's `select!` macro to impose a timeout.
pub struct ResponseAwaiter(oneshot::Receiver<rpc::Response>);

impl Future for ResponseAwaiter {
    type Output = Result<Option<Value>, ResponseError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            // we received a response, filter between a real result or an RPC error
            Poll::Ready(Ok(response)) => Poll::Ready(match response.error {
                Some(e) => Err(ResponseError::Rpc(e)),
                None => Ok(response.result),
            }),

            // no response received, the channel errored
            Poll::Ready(Err(error)) => Poll::Ready(Err(ResponseError::Recv(error))),

            // we are still waiting
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A response error. Either an RPC error (`rpc::Error`), or a receive error (`oneshot::error::RecvError`).
#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("rpc error")]
    Rpc(rpc::Error),

    #[error("receive error")]
    Recv(#[from] oneshot::error::RecvError),
}

pub struct Omegga {
    pub awaiter_txs: Arc<DashMap<rpc::RequestId, oneshot::Sender<rpc::Response>>>,
    request_id: Arc<AtomicI32>,
}

impl Omegga {
    /// Create a new Omegga instance.
    pub fn new() -> Self {
        Self {
            awaiter_txs: Arc::new(DashMap::new()),
            request_id: Arc::new(AtomicI32::new(-1)),
        }
    }

    /// Spawn the listener.
    pub fn spawn(&self) -> RpcEventReceiver {
        let (tx, rx) = mpsc::unbounded_channel::<rpc::Message>();
        let awaiter_txs = Arc::clone(&self.awaiter_txs);
        tokio::spawn(async move {
            let reader = BufReader::new(stdin());
            let mut lines = reader.lines();
            while let Some(line) = lines.next_line().await.unwrap() {
                let message: rpc::Message = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                match message {
                    // Handle responses
                    rpc::Message::Response {
                        id, result, error, ..
                    } => {
                        if let Entry::Occupied(entry) = awaiter_txs.entry(id) {
                            let (id, sender) = entry.remove_entry();
                            let _ = sender.send(rpc::Response { id, result, error });
                        }
                    }
                    // Otherwise, send everything else
                    _ => {
                        let _ = tx.send(message);
                    }
                };
            }
        });
        rx
    }

    /// Write out an RPC message.
    pub fn write(&self, message: rpc::Message) {
        println!("{}", serde_json::to_string(&message).unwrap());
    }

    /// Write out an RPC notification.
    pub fn write_notification(&self, method: impl Into<String>, params: Option<Value>) {
        self.write(rpc::Message::notification(method.into(), params));
    }

    /// Write out an RPC response.
    pub fn write_response(
        &self,
        id: rpc::RequestId,
        params: Option<Value>,
        error: Option<rpc::Error>,
    ) {
        self.write(rpc::Message::response(id, params, error));
    }

    /// Write out an RPC request.
    ///
    /// **Note:** This does not internally expect a response from the server.
    /// Prefer using [`request`](Omegga::request) over this for the ability to
    /// await a response from the RPC server.
    pub fn write_request(
        &self,
        id: rpc::RequestId,
        method: impl Into<String>,
        params: Option<Value>,
    ) {
        self.write(rpc::Message::request(id, method.into(), params));
    }

    /// Request a response from the RPC server.
    /// This returns a `ResponseAwaiter`, a `Future` that awaits a response.
    pub fn request(&self, method: impl Into<String>, params: Option<Value>) -> ResponseAwaiter {
        // fetch the next ID
        let id = self.request_id.fetch_sub(-1, Ordering::SeqCst);

        // write out the request
        self.write_request(rpc::RequestId::Int(id), method, params);

        // create a channel to send the response over
        let (tx, rx) = oneshot::channel::<rpc::Response>();

        // insert the transmitter into the dashmap
        self.awaiter_txs.insert(rpc::RequestId::Int(id), tx);

        // return back with an awaiter to await the receiver
        ResponseAwaiter(rx)
    }

    /// Prints a message to the Omegga console.
    pub fn log(&self, line: impl Into<String>) {
        self.write_notification("log", Some(Value::String(line.into())));
    }

    /// Prints a message to the Omegga console in error color.
    pub fn error(&self, line: impl Into<String>) {
        self.write_notification("error", Some(Value::String(line.into())));
    }

    /// Prints a message to the Omegga console in info color.
    pub fn info(&self, line: impl Into<String>) {
        self.write_notification("info", Some(Value::String(line.into())));
    }

    /// Prints a message to the Omegga console in warn color.
    pub fn warn(&self, line: impl Into<String>) {
        self.write_notification("warn", Some(Value::String(line.into())));
    }

    /// Prints a message to the Omegga console in trace color.
    pub fn trace(&self, line: impl Into<String>) {
        self.write_notification("trace", Some(Value::String(line.into())));
    }

    /// Gets an object from the store.
    pub async fn store_get(&self, key: impl Into<String>) -> Result<Option<Value>, ResponseError> {
        self.request("store.get", Some(Value::String(key.into())))
            .await
    }

    /// Sets an object in the store.
    pub fn store_set(&self, key: impl Into<String>, value: Value) {
        self.write_notification("store.set", Some(json!([key.into(), value])))
    }

    /// Deletes an object from the store.
    pub async fn store_delete(&self, key: impl Into<String>) {
        self.write_notification("store.delete", Some(Value::String(key.into())))
    }

    /// Wipes the store.
    pub fn store_wipe(&self) {
        self.write_notification("store.wipe", None)
    }

    /// Gets a list of keys in the store.
    pub async fn store_keys(&self) -> Result<Vec<String>, ResponseError> {
        self.request("store.keys", None).await.map(|r| match r {
            Some(r) => serde_json::from_value::<Vec<String>>(r).unwrap_or_else(|_| vec![]),
            None => vec![],
        })
    }

    /// Writes a line out to the Brickadia server.
    pub fn writeln(&self, line: impl Into<String>) {
        self.write_notification("exec", Some(Value::String(line.into())));
    }

    /// Broadcasts a line.
    pub fn broadcast(&self, line: impl Into<String>) {
        self.write_notification("broadcast", Some(Value::String(line.into())));
    }

    /// Whispers a line to a user by their name.
    pub fn whisper(&self, username: impl Into<String>, line: impl Into<String>) {
        self.write_notification(
            "whisper",
            Some(json!({"target": username.into(), "line": line.into()})),
        );
    }

    /// Gets a list of all players.
    pub async fn get_players(&self) -> Result<Vec<Player>, ResponseError> {
        self.request("getPlayers", None).await.map(|r| match r {
            Some(r) => serde_json::from_value::<Vec<Player>>(r).unwrap_or_else(|_| vec![]),
            None => vec![],
        })
    }

    /// Get all player positions.
    pub async fn get_all_player_positions(&self) -> Result<Vec<PlayerPosition>, ResponseError> {
        self.request("getAllPlayerPositions", None)
            .await
            .map(|r| match r {
                Some(r) => {
                    serde_json::from_value::<Vec<PlayerPosition>>(r).unwrap_or_else(|_| vec![])
                }
                None => vec![],
            })
    }

    /// Get the role setup.
    pub async fn get_role_setup(&self) -> Result<Value, ResponseError> {
        // TODO: write a type for this instead of using a serde_json::Value
        self.request("getRoleSetup", None).await.map(Option::unwrap)
    }

    /// Get the ban list.
    pub async fn get_ban_list(&self) -> Result<Value, ResponseError> {
        // TODO: write a type for this instead of using a serde_json::Value
        self.request("getBanList", None).await.map(Option::unwrap)
    }

    /// Get a list of the server's saves.
    pub async fn get_saves(&self) -> Result<Vec<String>, ResponseError> {
        self.request("getSaves", None).await.map(|r| match r {
            Some(r) => serde_json::from_value::<Vec<String>>(r).unwrap_or_else(|_| vec![]),
            None => vec![],
        })
    }

    /// Get the path to a specific save.
    pub async fn get_save_path(
        &self,
        save: impl Into<String>,
    ) -> Result<Option<String>, ResponseError> {
        self.request("getSavePath", Some(Value::String(save.into())))
            .await
            .map(|r| match r {
                Some(r) => serde_json::from_value::<String>(r).ok(),
                None => None,
            })
    }

    /// Gets the server's current save data.
    #[cfg(not(feature = "brs"))]
    pub async fn get_save_data(&self) -> Result<Value, ResponseError> {
        self.request("getSaveData", None).await.map(Option::unwrap)
    }

    /// Gets the server's current save data as a brickadia-rs save object.
    #[cfg(feature = "brs")]
    pub async fn get_save_data(&self) -> Result<save::SaveData, ResponseError> {
        self.request("getSaveData", None)
            .await
            .map(|r| serde_json::from_value::<save::SaveData>(r.unwrap()).unwrap())
    }

    /// Clears a player's bricks by their name.
    pub fn clear_bricks(&self, target: impl Into<String>, quiet: bool) {
        self.write_notification(
            "clearBricks",
            Some(json!({"target": target.into(), "quiet": quiet})),
        );
    }

    /// Clear all bricks.
    pub fn clear_all_bricks(&self, quiet: bool) {
        self.write_notification("clearAllBricks", Some(json!({ "quiet": quiet })));
    }

    /// Save bricks to a named save.
    pub async fn save_bricks(&self, name: impl Into<String>) -> Result<(), ResponseError> {
        self.request("saveBricks", Some(Value::String(name.into())))
            .await
            .map(|_| ())
    }

    /// Load a save, provided an offset in the world.
    pub async fn load_bricks(
        &self,
        name: impl Into<String>,
        quiet: bool,
        offset: (i32, i32, i32),
    ) -> Result<(), ResponseError> {
        self.request("loadBricks", Some(json!({"name": name.into(), "quiet": quiet, "offX": offset.0, "offY": offset.1, "offZ": offset.2}))).await.map(|_| ())
    }

    /// Reads a save (from a save file), and returns its data.
    #[cfg(not(feature = "brs"))]
    pub async fn read_save_data(
        &self,
        name: impl Into<String>,
    ) -> Result<Option<Value>, ResponseError> {
        self.request("readSaveData", Some(Value::String(name.into())))
            .await
    }

    /// Reads a save (from a save file), and returns its data as a brickadia-rs save object.
    #[cfg(feature = "brs")]
    pub async fn read_save_data(
        &self,
        name: impl Into<String>,
    ) -> Result<Option<save::SaveData>, ResponseError> {
        self.request("readSaveData", Some(Value::String(name.into())))
            .await
            .map(|r| match r {
                Some(r) => serde_json::from_value::<save::SaveData>(r).ok(),
                None => None,
            })
    }

    /// Loads a save (from a JSON value) into the world, provided an offset.
    #[cfg(not(feature = "brs"))]
    pub async fn load_save_data(
        &self,
        data: Value,
        quiet: bool,
        offset: (i32, i32, i32),
    ) -> Result<(), ResponseError> {
        self.request("loadSaveData", Some(json!({"data": data, "quiet": quiet, "offX": offset.0, "offY": offset.1, "offZ": offset.2}))).await.map(|_| ())
    }

    /// Loads a save (from brickadia-rs save data) into the world, provided an offset.
    #[cfg(feature = "brs")]
    pub async fn load_save_data(
        &self,
        data: save::SaveData,
        quiet: bool,
        offset: (i32, i32, i32),
    ) -> Result<(), ResponseError> {
        self.request("loadSaveData", Some(json!({"data": data, "quiet": quiet, "offX": offset.0, "offY": offset.1, "offZ": offset.2}))).await.map(|_| ())
    }

    /// Changes the map.
    pub async fn change_map(&self, map: impl Into<String>) -> Result<(), ResponseError> {
        self.request("changeMap", Some(Value::String(map.into())))
            .await
            .map(|_| ())
    }

    /// Get a player.
    pub async fn get_player(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<Player>, ResponseError> {
        self.request("player.get", Some(Value::String(target.into())))
            .await
            .map(|r| serde_json::from_value::<Player>(r.unwrap_or(Value::Null)).ok())
    }

    /// Get a player's roles.
    pub async fn get_player_roles(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<Vec<String>>, ResponseError> {
        self.request("player.getRoles", Some(Value::String(target.into())))
            .await
            .map(|r| serde_json::from_value::<Vec<String>>(r.unwrap_or(Value::Null)).ok())
    }

    /// Get a player's permissions.
    pub async fn get_player_permissions(
        &self,
        target: impl Into<String>,
    ) -> Result<Value, ResponseError> {
        self.request("player.getPermissions", Some(Value::String(target.into())))
            .await
            .map(|r| r.unwrap_or(Value::Null))
    }

    /// Get a player's name color (6-digit hexadecimal).
    pub async fn get_player_name_color(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<String>, ResponseError> {
        self.request("player.getNameColor", Some(Value::String(target.into())))
            .await
            .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Get a player's position.
    pub async fn get_player_position(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<(f64, f64, f64)>, ResponseError> {
        self.request("player.getPosition", Some(Value::String(target.into())))
            .await
            .map(|r| match r {
                Some(r) => serde_json::from_value::<(f64, f64, f64)>(r).ok(),
                None => None,
            })
    }

    /// Get a player's ghost brick data.
    pub async fn get_player_ghost_brick(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<GhostBrick>, ResponseError> {
        self.request("player.getGhostBrick", Some(Value::String(target.into())))
            .await
            .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Get a player's paint data.
    pub async fn get_player_paint(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<PlayerPaint>, ResponseError> {
        self.request("player.getPaint", Some(Value::String(target.into())))
            .await
            .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Get a player's template bounds.
    pub async fn get_player_template_bounds(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<TemplateBounds>, ResponseError> {
        self.request(
            "player.getTemplateBounds",
            Some(Value::String(target.into())),
        )
        .await
        .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Get a player's template data.
    #[cfg(not(feature = "brs"))]
    pub async fn get_player_template_bounds_data(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<Value>, ResponseError> {
        self.request(
            "player.getTemplateBoundsData",
            Some(Value::String(target.into())),
        )
        .await
    }

    /// Get a player's template data as a brickadia-rs save object.
    #[cfg(feature = "brs")]
    pub async fn get_player_template_bounds_data(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<save::SaveData>, ResponseError> {
        self.request(
            "player.getTemplateBoundsData",
            Some(Value::String(target.into())),
        )
        .await
        .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Load save data at a player's template.
    #[cfg(not(feature = "brs"))]
    pub async fn load_data_at_ghost_brick(
        &self,
        target: impl Into<String>,
        data: Value,
        offset: (i32, i32, i32),
        rotate: bool,
        quiet: bool,
    ) -> Result<(), ResponseError> {
        self.request("player.loadDataAtGhostBrick", Some(json!({"target": target.into(), "data": data, "offX": offset.0, "offY": offset.1, "offZ": offset.2, "rotate": rotate, "quiet": quiet})))
            .await
            .map(|_| ())
    }

    /// Load brickadia-rs save data at a player's template.
    #[cfg(feature = "brs")]
    pub async fn load_data_at_ghost_brick(
        &self,
        target: impl Into<String>,
        data: save::SaveData,
        offset: (i32, i32, i32),
        rotate: bool,
        quiet: bool,
    ) -> Result<(), ResponseError> {
        self.request("player.loadDataAtGhostBrick", Some(json!({"target": target.into(), "data": data, "offX": offset.0, "offY": offset.1, "offZ": offset.2, "rotate": rotate, "quiet": quiet})))
            .await
            .map(|_| ())
    }

    /// Get a plugin.
    pub async fn get_plugin(
        &self,
        target: impl Into<String>,
    ) -> Result<Option<Plugin>, ResponseError> {
        self.request("plugin.get", Some(Value::String(target.into())))
            .await
            .map(|r| r.and_then(|r| serde_json::from_value::<_>(r).ok()))
    }

    /// Emit a custom event to a plugin.
    pub async fn emit_plugin<T>(
        &self,
        target: impl Into<String>,
        event: impl Into<String>,
        args: Vec<Value>,
    ) -> Result<Option<T>, ResponseError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut query = vec![Value::String(target.into()), Value::String(event.into())];
        query.extend(args.into_iter());

        self.request("plugin.emit", Some(Value::Array(query)))
            .await
            .map(|r| {
                serde_json::from_value::<_>(r.unwrap_or_default())
                    .ok()
                    .unwrap_or_default()
            })
    }
}

impl Default for Omegga {
    fn default() -> Self {
        Self::new()
    }
}
