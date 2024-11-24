use anyhow::Result;
use bytes::BytesMut;
use course_proto::pb::abi::{CommandRequest, CommandResponse};
use futures::prelude::*;
use prost::Message;
use tokio::net::TcpListener;
use tracing::info;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:3333";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);

        tokio::spawn(async move {
            let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

            while let Some(Ok(bytes)) = framed.next().await {
                let msg = CommandRequest::decode(&bytes[..])?;
                info!("Got a new command: {:?}", msg);
                
                let mut resp = CommandResponse::default();
                resp.status = 404;
                resp.message = "Not Found".to_string();
                
                let mut buf = BytesMut::new();
                resp.encode(&mut buf)?;
                framed.send(buf.freeze()).await?;
            }
            info!("Client {:?} disconnected", addr);
            Ok::<_, anyhow::Error>(())
        });
    }
}
