use tokio::sync::Mutex;

#[derive(Debug)]
pub struct BufferPool {
    pool: Mutex<Vec<bytes::BytesMut>>,
    max_pool_size: usize,
    buf_capacity: usize,
}

impl BufferPool {
    /// Creates a new buffer pool with the specified maximum pool size and buffer capacity.
    /// The pool is initially empty and will grow up to the maximum pool size as buffers are released.
    pub fn new(max_pool_size: usize, buf_capacity: usize) -> Self {
        let pool = Vec::with_capacity(max_pool_size);
        Self {
            pool: Mutex::new(pool),
            buf_capacity,
            max_pool_size,
        }
    }

    /// Returns the buffer capacity used by the pool.
    pub fn buf_capacity(&self) -> usize {
        self.buf_capacity
    }

    /// Acquires a buffer from the pool. If the pool is empty, a new buffer with the specified capacity is created.
    pub async fn acquire(&self) -> bytes::BytesMut {
        let mut pool = self.pool.lock().await;
        if let Some(buffer) = pool.pop() {
            buffer
        } else {
            bytes::BytesMut::with_capacity(self.buf_capacity)
        }
    }

    /// Releases a buffer back to the pool.
    /// The buffer is dropped if the pool has reached its maximum size or if it does not have the desired capacity.
    pub async fn release(&self, mut buffer: bytes::BytesMut) {
        buffer.clear();
        if buffer.try_reclaim(self.buf_capacity) {
            let mut pool = self.pool.lock().await;
            if pool.len() < self.max_pool_size {
                // Only push the buffer back to the pool if it has the desired capacity.
                // Buffers from long messages are dropped
                pool.push(buffer);
            }
        }
    }

    pub async fn with_buffer<F, R>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut bytes::BytesMut) -> R,
    {
        let mut buffer = self.acquire().await;
        let result = f(&mut buffer).await;
        self.release(buffer).await;
        result
    }
}
