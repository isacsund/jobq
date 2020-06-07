use crate::{ClientMessage, ServerMessage, ToMpart};
use anyhow::Error;
use futures::{Sink, SinkExt, StreamExt, TryStreamExt};
use tmq::{router, Context, Message, Multipart};

pub struct Server {
    bind: String,
}

impl Server {
    pub fn new(bind: String) -> Self {
        Server { bind }
    }
}

impl Server {
    pub async fn run(&self) -> Result<(), Error> {
        let (mut send, mut recv) = router(&Context::new())
            .bind(&self.bind)?
            .split::<Multipart>();

        while let Some(msg) = recv.try_next().await? {
            let client_name = &msg[0];
            let server_msg = serde_cbor::from_slice::<ServerMessage>(&msg[1]);

            match server_msg {
                Ok(ServerMessage::Hello) => {
                    if let Some(name) = client_name.as_str() {
                        println!("New connection opened from: {}", name);
                    }

                    send.send(
                        vec![
                            Message::from(&client_name as &[u8]),
                            ClientMessage::Hello.to_msg()?,
                        ]
                        .into(),
                    )
                    .await?;
                }
                Ok(ServerMessage::Request(job_request)) => {
                    println!("{:?}", job_request);
                }
                Ok(_) => {
                    println!("Unkown message");
                }
                Err(err) => {
                    println!("Could not deserialize message:{}", err);
                }
            }
        }
        Ok(())
    }
}
