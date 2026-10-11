use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use crate::msg::binary::{DecodeBinary, EncodeBinary};
use crate::msg::uacp::Msg;

pub struct Connection {
    stream: net::TcpStream,
    send_buf: BytesMut,
    recv_buf: BytesMut,
}

impl crate::Connection for Connection {
    async fn connect(endpoint: &str) -> Result<Self, super::Error> {
        if !endpoint.starts_with("opc.tcp://") {
            return Err(super::Error::UnsupportedProtocol(endpoint.to_string()));
        }
        let endpoint = &endpoint[10..]; // Remove "opc.tcp://"
        let stream = net::TcpStream::connect(endpoint).await?;
        let send_buf = BytesMut::with_capacity(8192);
        let recv_buf = BytesMut::with_capacity(8192);
        Ok(Connection {
            stream,
            send_buf,
            recv_buf,
        })
    }

    async fn send(&mut self, msg: Msg) -> Result<(), super::Error> {
        self.send_buf.clear();
        msg.encode(&mut self.send_buf);
        println!("Sending message of length: {}", self.send_buf.len());
        let data = self.send_buf.split().freeze();
        self.stream.write_all(&data).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Msg, super::Error> {
        self.recv_buf.clear();
        let n = self.stream.read_buf(&mut self.recv_buf).await?;
        if n == 0 {
            return Err(super::Error::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed",
            )));
        }
        
        let mut buf = &self.recv_buf[..];
        let msg = Msg::decode(&mut buf)?;
        Ok(msg)
    }
}
