use bytes::Buf;
use tokio::io;

use crate::ua;

pub mod tcp;
pub mod uacp;

#[allow(async_fn_in_trait)]
pub trait SendData {
    async fn send_data(&mut self, data: &bytes::BytesMut) -> Result<(), io::Error>;
}

pub trait RecvData {
    fn recv_data(&mut self, data: &mut bytes::BytesMut) -> impl Future<Output = Result<usize, io::Error>> + Send;
}

pub trait SplitTransport {
    type Send: SendData;
    type Recv: RecvData;
}


#[derive(Debug)]
pub struct SendChunk<S> {
    inner: S,
    max_chunk_size: usize,
}

impl<S: SendData> SendChunk<S> {
    pub fn new(inner: S, max_chunk_size: usize) -> Self {
        Self {
            inner,
            max_chunk_size,
        }
    }

    pub fn adjust_max_chunk_size(&mut self, new_max_chunk_size: usize) {
        self.max_chunk_size = new_max_chunk_size;
    }

    pub async fn send_chunk(&mut self, buf: &bytes::BytesMut) -> Result<(), ua::StatusCode> {
        self.check_size(buf)?;
        self.inner
            .send_data(buf)
            .await
            .map_err(|_| ua::StatusCode::BadCommunicationError)
    }

    fn check_size(&self, buf: &bytes::BytesMut) -> Result<(), ua::StatusCode> {
        if buf.remaining() < 8 {
            return Err(ua::StatusCode::BadEncodingError);
        }
        let msg_len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if msg_len != buf.remaining() as u32 {
            return Err(ua::StatusCode::BadEncodingError);
        }
        if msg_len as usize > self.max_chunk_size {
            return Err(ua::StatusCode::BadTcpMessageTooLarge);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RecvChunk<S> {
    inner: S,
    buf: bytes::BytesMut,
    max_chunk_size: usize,
}

impl<S: RecvData> RecvChunk<S> {
    pub fn new(inner: S, max_chunk_size: usize) -> Self {
        Self {
            inner,
            buf: bytes::BytesMut::new(),
            max_chunk_size,
        }
    }

    pub fn adjust_max_chunk_size(&mut self, new_max_chunk_size: usize) {
        self.max_chunk_size = new_max_chunk_size;
        if self.buf.len() > self.max_chunk_size {
            self.buf.truncate(self.max_chunk_size);
        }
    }

    pub async fn recv_chunk(&mut self, buf: &mut bytes::BytesMut) -> Result<(), ua::StatusCode> {
        debug_assert!(buf.is_empty());
        debug_assert!(buf.capacity() >= self.max_chunk_size);

        self.fill_up_to(buf, 8).await?;

        let msg_len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if msg_len as usize > self.max_chunk_size {
            return Err(ua::StatusCode::BadTcpMessageTooLarge);
        }

        self.fill_up_to(buf, msg_len as usize).await?;

        if buf.len() > msg_len as usize {
            self.buf.extend_from_slice(&buf[msg_len as usize..]);
            buf.truncate(msg_len as usize);
        }

        Ok(())
    }

    async fn fill_up_to(
        &mut self,
        buf: &mut bytes::BytesMut,
        n: usize,
    ) -> Result<(), ua::StatusCode> {
        if self.buf.len() > 0 {
            let copied_len = std::cmp::min(self.buf.len(), n - buf.len());
            buf.extend_from_slice(&self.buf[..copied_len]);
            if n < self.buf.len() {
                self.buf.copy_within(copied_len.., 0);
                self.buf.truncate(self.buf.len() - copied_len);
            }
        }
        while buf.len() < n {
            let m = self
                .inner
                .recv_data(buf)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            if m == 0 {
                return Err(ua::StatusCode::BadConnectionClosed);
            }
        }
        Ok(())
    }
}
