use crate::{ClientMessage, Job, ServerMessage, ToMpart, WorkerMessage};
use anyhow::Error;
use futures::{Sink, SinkExt, StreamExt, TryStreamExt};
use tmq::TmqError;
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

        let mut job: Option<Job> = None;
        let mut active: u64 = 0;

        while let Some(msg) = recv.try_next().await? {
            let client_name = &msg[0];
            let server_msg = serde_cbor::from_slice::<ServerMessage>(&msg[1]);

            match server_msg {
                Ok(ServerMessage::Hello) => {
                    if let Some(name) = client_name.as_str() {
                        println!("ping: {}", name);
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

                    job = Some(Job {
                        id: 1,
                        name: job_request.name,
                        username: job_request.username,
                    })
                }
                Ok(_) => {
                    println!("Unkown message");
                }
                Err(err) => {
                    println!("Could not deserialize message:{}", err);
                }
            }

            if let Some(j) = &job {
                send_job(j.clone(), &mut send).await?;
                job = None;
            }
        }
        Ok(())
    }
}

async fn send_job<S: Sink<Multipart, Error = TmqError> + Unpin>(
    job: Job,
    send: &mut S,
) -> Result<(), Error> {
    send.send(
        vec![
            Message::from(job.name.as_bytes()),
            WorkerMessage::Order(job).to_msg()?,
        ]
        .into(),
    )
    .await?;
    Ok(())
}
