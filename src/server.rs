use anyhow::Error;
use futures::{StreamExt, TryStreamExt};
use tmq::{router, Context, Multipart};

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
            println!("{:?}", msg);
        }
        Ok(())
    }
}
