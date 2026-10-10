use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net;

use crate::msg::binary::EncodeBinary;

pub struct Connection {
    stream: net::TcpStream,
    buf: BytesMut,
}

impl crate::Connection for Connection {
    async fn connect(endpoint: &str) -> Result<Self, super::Error> {
        if !endpoint.starts_with("opc.tcp://") {
            return Err(super::Error::UnsupportedProtocol(endpoint.to_string()));
        }
        let endpoint = &endpoint[10..]; // Remove "opc.tcp://"
        let stream = net::TcpStream::connect(endpoint).await?;
        let buf = BytesMut::with_capacity(8192);
        Ok(Connection { stream, buf })
    }

    async fn send(&mut self, msg: impl EncodeBinary) -> Result<(), super::Error> {
        self.buf.clear();
        msg.encode(&mut self.buf);
        println!("Sending message of length: {}", self.buf.len());
        let data = self.buf.split().freeze();
        self.stream.write_all(&data).await?;
        Ok(())
    }
}
