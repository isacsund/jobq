use anyhow::{anyhow, Error};
use futures::{SinkExt, StreamExt, TryStream, TryStreamExt};
use tmq::{dealer, Context, Multipart, TmqError};

use jobq::settings::Settings;
use jobq::{ClientMessage, JobRequest, ServerMessage, ToMpart};

async fn get_message<S: TryStream<Ok = Multipart, Error = TmqError> + Unpin>(
    recv: &mut S,
) -> Result<ClientMessage, Error> {
    if let Some(msg) = recv.try_next().await? {
        let jobq_message: ClientMessage = serde_cbor::from_slice(&msg[0])?;

        Ok(jobq_message)
    } else {
        Err(anyhow!("No Messages in Stream"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let settings = Settings::get();

    let (mut send, mut recv) = dealer(&Context::new())
        .set_identity(b"test_client")
        .connect(&settings.server.bind)?
        .split::<Multipart>();

    // Send hello
    send.send(ClientMessage::Hello.to_mpart()?).await?;

    if let ClientMessage::Hello = get_message(&mut recv).await? {
        println!("Received Hello response, sending a couple of jobs");

        for i in 0..500 {
            let job = JobRequest {
                name: "test".into(),
                username: "test_client".into(),
            };

            send.send(ServerMessage::Request(job).to_mpart()?).await?;
        }

        println!("Done!");
    }

    loop {
        let message = get_message(&mut recv).await?;

        println!("Message:{:?}", message);
    }
}
