#[macro_use]
pub extern crate diesel;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tmq::{Message, Multipart};

pub mod schema;
pub mod server;
pub mod settings;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Hello,
    Request(JobRequest),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Hello,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRequest {
    pub name: String,
    pub username: String,
}

pub trait ToMpart {
    fn to_mpart(&self) -> Result<Multipart, Error>;

    fn to_msg(&self) -> Result<Message, Error>;
}

impl<T: serde::ser::Serialize> ToMpart for T {
    fn to_mpart(&self) -> Result<Multipart, Error> {
        let bytes = serde_cbor::to_vec(&self)?;

        Ok(Multipart::from(vec![&bytes]))
    }

    fn to_msg(&self) -> Result<Message, Error> {
        let bytes = serde_cbor::to_vec(&self)?;

        Ok(Message::from(&bytes))
    }
}
