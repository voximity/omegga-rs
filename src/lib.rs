use rpc::RequestId;
use smol::lock::MutexGuard;

use {
    rpc::{RpcMessage, RpcError},
    std::{collections::HashMap, sync::Arc, time::Duration},
    smol::{Timer, Unblock, future::FutureExt, lock::Mutex, stream::StreamExt, channel::{self, Sender, Receiver}, io::{self, AsyncBufReadExt}}
};

pub use serde_json;
pub use smol;

use serde_json::Value;

pub mod rpc;

pub struct Omegga {
    channels_ref: Arc<Mutex<HashMap<rpc::RequestId, Sender<Result<Value, RpcError>>>>>,

    last_id: i32
}

pub struct OmeggaWrapper {
    omegga: Arc<Mutex<Omegga>>,

    // receive and respond to events from the user's side
    pub stream: Arc<Receiver<RpcMessage>>,

    // internal stream parts
    int_stream_sender: Arc<Sender<RpcMessage>>
}

impl OmeggaWrapper {
    pub fn new() -> Self {
        // stream to user, response from user
        let (int_stream_sender, stream_receiver) = channel::unbounded();

        OmeggaWrapper {
            omegga: Arc::new(Mutex::new(Omegga::new())),
            stream: Arc::new(stream_receiver), int_stream_sender: Arc::new(int_stream_sender)
        }
    }

    pub async fn inner(&self) -> MutexGuard<'_, Omegga> {
        self.omegga.lock().await
    }

    pub fn clone_inner(&self) -> Arc<Mutex<Omegga>> {
        self.omegga.clone()
    }

    pub fn start(&mut self) {
        smol::block_on(async {
            let reader = io::BufReader::new(Unblock::new(std::io::stdin()));
            let mut lines = reader.lines();

            while let Some(line) = lines.next().await {
                let string = line.expect("Unable to fetch from stdin");

                // parse into an RpcMessage
                let message: RpcMessage = match serde_json::from_str(string.as_str()) {
                    Ok(m) => m,
                    Err(_) => continue
                };

                // match the message and let the handlers know
                // in a new worker
                let self_locked = self.omegga.lock_arc().await;
                let sender_locked = self.int_stream_sender.clone();
                let channels = self_locked.channels_ref.clone();
                smol::spawn(async move {
                    match message {
                        RpcMessage::Response { id, result, error, .. } => {
                            let channels = &mut channels.lock().await;
                            match channels.get(&id) {
                                Some(sender) => {
                                    let res = match result {
                                        Some(value) => Ok(value),
                                        None => Err(error.expect("An error object from RPC was expected"))
                                    };

                                    // at this point, send the result over the sender
                                    // we can also remove this entry from the hashtable
                                    sender.send(res).await.unwrap();
                                    channels.remove(&id);
                                },
                                None => ()
                            }
                        },
                        _ => sender_locked.send(message).await.unwrap()
                    }
                }).detach();
            }
        });
    }
}

impl Omegga {
    pub fn new() -> Self {
        Omegga { channels_ref: Arc::new(Mutex::new(HashMap::new())), last_id: 0 }
    }

    /// Notify Omegga, given a method and some parameter.
    pub fn notify(method: &str, params: Value) {
        println!("{}", serde_json::to_string(&RpcMessage::notification(method.into(), Some(params))).unwrap());
    }

    /// Notify Omegga with no parameter.
    pub fn tell(method: &str) {
        println!("{}", serde_json::to_string(&RpcMessage::notification(method.into(), None)).unwrap());
    }

    pub fn respond(id: RequestId, body: Result<Value, RpcError>) {
        let response = match body {
            Ok(value) => RpcMessage::response(id, Some(value), None),
            Err(err) => RpcMessage::response(id, None, Some(err))
        };

        println!("{}", serde_json::to_string(&response).unwrap());
    }

    pub async fn request(mutex: Arc<Mutex<Self>>, method: &str, params: Option<Value>) -> ResponseAwaiter {
        let local_self = &mut mutex.lock().await;

        local_self.last_id -= 1;
        let request = RpcMessage::request(rpc::RequestId::Int(local_self.last_id), String::from(method), params);
        println!("{}", serde_json::to_string(&request).unwrap());

        let (rx, tx) = channel::bounded::<Result<Value, RpcError>>(1);
        let channels = Arc::clone(&local_self.channels_ref);

        {
            let mut mtx = channels.lock().await;
            mtx.insert(rpc::RequestId::Int(local_self.last_id), rx);
        }

        ResponseAwaiter::new(tx)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ResponseReceiveError {
    #[error("channel receive error")]
    Recv(#[from] smol::channel::RecvError),
    #[error("timed out")]
    Timeout
}

pub struct ResponseAwaiter {
    receiver: Receiver<Result<Value, RpcError>>
}

impl ResponseAwaiter {
    pub fn new(receiver: Receiver<Result<Value, RpcError>>) -> Self {
        ResponseAwaiter { receiver }
    }

    pub async fn receive(self) -> Result<Result<Value, RpcError>, ResponseReceiveError> {
        async move { Ok(self.receiver.recv().await?) }
            .or(async {
                Timer::after(Duration::from_secs(15)).await;
                Err(ResponseReceiveError::Timeout)
            }).await
    }
}
