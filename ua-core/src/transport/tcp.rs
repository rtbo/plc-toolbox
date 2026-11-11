use tokio::io;
use tokio::net;

use super::{RecvData, SendData};

pub type SendChunk = super::SendChunk<net::tcp::OwnedWriteHalf>;
pub type RecvChunk = super::RecvChunk<net::tcp::OwnedReadHalf>;

impl SendData for net::tcp::OwnedWriteHalf {
    async fn send_data(&mut self, data: &bytes::BytesMut) -> Result<(), io::Error> {
        use io::AsyncWriteExt;
        self.write_all(&data).await?;
        Ok(())
    }
}

impl RecvData for net::tcp::OwnedReadHalf {
    async fn recv_data(&mut self, data: &mut bytes::BytesMut) -> Result<usize, io::Error> {
        use io::AsyncReadExt;
        Ok(self.read_buf(data).await?)
    }
}
