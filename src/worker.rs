use crate::{Job, ServerMessage, ToMpart, WorkerMessage};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use futures::{channel::mpsc::unbounded, future::ready, SinkExt, StreamExt};
use std::time::Duration;
use tmq::{dealer, Context, Multipart};
use tokio::time::delay_for;

#[async_trait]
pub trait Worker: Sized {
    const JOB_NAME: &'static str;

    async fn process(&self, job: Job) -> Result<(), Error>;

    async fn work(&self, job_address: &str) -> Result<(), Error> {
        let job_type = Self::JOB_NAME;
        println!("Worker: {} starting, sending hello", job_type);

        let (mut send_socket, recv) = dealer(&Context::new())
            .set_identity(job_type.as_bytes())
            .connect(&job_address)?
            .split::<Multipart>();

        // Create a unbounded channel for messages
        let (send, mut recv_socket) = unbounded::<ServerMessage>();

        // Spawn a new task for fetching messages from the unbounded
        // channel reciever and sending them forward to the socket server
        tokio::spawn(async move {
            while let Some(jobq_message) = recv_socket.next().await {
                if let Ok(msg) = jobq_message.to_mpart() {
                    if let Err(err) = send_socket.send(msg).await {
                        println!("Error sending message: {}", err);
                    }
                }
            }
        });

        let mut ping_sender = send.clone();

        // Spawn a new task for sending pings to the unbounded channel
        tokio::spawn(async move {
            loop {
                if let Err(err) = ping_sender.send(ServerMessage::Hello).await {
                    println!("Error: {}", err);
                }
                delay_for(Duration::from_secs(5)).await;
            }
        });

        recv.filter_map(|val| {
            match val
                .map_err(Error::from)
                .and_then(|msg| serde_cbor::from_slice(&msg[0]).map_err(Error::from))
            {
                Ok(WorkerMessage::Order(job)) => {
                    println!("Job recieved");
                    return ready(Some(job));
                }
                Ok(WorkerMessage::Hello) => {
                    println!("pong: {}", job_type);
                }
                Err(err) => {
                    println!("Error decoding message: {}", err);
                }
            }

            return ready(None);
        })
        .map(|job| (self.process(job.clone()), send.clone(), job))
        .for_each_concurrent(None, |(status, mut send, job)| async move {
            let server_message = match status.await {
                Ok(()) => println!("Job done"),
                Err(err) => println!("Job failed: {}", err),
            };
        })
        .await;

        Ok(())
    }
}

pub struct TestWorker;

#[async_trait]
impl Worker for TestWorker {
    const JOB_NAME: &'static str = "test_worker";

    async fn process(&self, job: Job) -> Result<(), Error> {
        delay_for(Duration::from_millis(100)).await;
        if job.id % 12 == 0 {
            return Err(anyhow!("Simulating failure"));
        }

        Ok(())
    }
}
