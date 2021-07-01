use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Str(String),
    Int(i32)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RpcMessage {
    Request { jsonrpc: String, id: RequestId, method: String, params: Option<Value> },
    Response { jsonrpc: String, id: RequestId, result: Option<Value>, error: Option<RpcError> },
    Notification { jsonrpc: String, method: String, params: Option<Value> }
}

impl RpcMessage {
    pub fn request(id: RequestId, method: String, params: Option<Value>) -> RpcMessage {
        RpcMessage::Request { jsonrpc: String::from("2.0"), id, method, params }
    }

    pub fn response(id: RequestId, result: Option<Value>, error: Option<RpcError>) -> RpcMessage {
        RpcMessage::Response { jsonrpc: String::from("2.0"), id, result, error }
    }

    pub fn notification(method: String, params: Option<Value>) -> RpcMessage {
        RpcMessage::Notification { jsonrpc: String::from("2.0"), method, params }
    }
}
