use anyhow::Result;
use bytes::BytesMut;
use course_proto::pb::abi::{CommandRequest, CommandResponse};
use futures::prelude::*;
use prost::Message;
use tokio::net::TcpStream;
use tracing::info;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:3333";
    let stream = TcpStream::connect(addr).await?;
    
    let mut client = Framed::new(stream, LengthDelimitedCodec::new());
    
    // 生成一个 HSET 命令
    let cmd = CommandRequest::new_hset("tb1", "language", "rakulang".into());
    
    // 序列化命令
    let mut buf = BytesMut::new();
    cmd.encode(&mut buf)?;
    
    // 发送命令
    client.send(buf.freeze()).await?;

    // 接收响应
    if let Some(Ok(buf)) = client.next().await {
        let resp = CommandResponse::decode(&buf[..])?;
        info!("Got response {:?}", resp);
    }

    Ok(())
}
