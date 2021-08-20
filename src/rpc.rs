use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An RPC error, as defined in the RPC specification.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// An RPC request ID. Can be a string (`Str`) or an integer (`Int`).
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum RequestId {
    Str(String),
    Int(i32),
}

impl From<String> for RequestId {
    fn from(str: String) -> Self {
        RequestId::Str(str)
    }
}

impl From<i32> for RequestId {
    fn from(i: i32) -> Self {
        RequestId::Int(i)
    }
}

/// An RPC message. One of [`Request`, `Response`, `Notification`].
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Message {
    Request {
        jsonrpc: String,
        id: RequestId,
        method: String,
        params: Option<Value>,
    },
    Response {
        jsonrpc: String,
        id: RequestId,
        result: Option<Value>,
        error: Option<Error>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        params: Option<Value>,
    },
}

impl Message {
    pub fn request(id: RequestId, method: String, params: Option<Value>) -> Self {
        Message::Request {
            jsonrpc: String::from("2.0"),
            id,
            method,
            params,
        }
    }

    pub fn response(id: RequestId, result: Option<Value>, error: Option<Error>) -> Self {
        Message::Response {
            jsonrpc: String::from("2.0"),
            id,
            result,
            error,
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Message::Notification {
            jsonrpc: String::from("2.0"),
            method,
            params,
        }
    }
}

/// A struct that contains the same data as `Message::Response`.
/// Used to save redundant `match`es against a `Message` that is
/// known to be a `Message::Response`.
#[derive(Debug, Clone)]
pub struct Response {
    pub id: RequestId,
    pub result: Option<Value>,
    pub error: Option<Error>,
}

impl TryFrom<Message> for Response {
    type Error = ();

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Response {
                id, result, error, ..
            } => Ok(Self { id, result, error }),
            _ => Err(()),
        }
    }
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        Self::Response {
            jsonrpc: "2.0".into(),
            id: value.id,
            result: value.result,
            error: value.error,
        }
    }
}
