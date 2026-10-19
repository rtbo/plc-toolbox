use ua_core::Transport;
use ua_core::transport::tcp;

pub type TcpClient = Client<tcp::Transport>;

pub struct Client<T> {
    transport: Option<T>,
}

impl<T> Client<T>
where
    T: Transport,
{
    pub fn new() -> Self {
        Client { transport: None }
    }

    pub async fn connect(&mut self, endpoint: &str) -> Result<(), ua_core::transport::Error> {
        use ua_core::msg::uacp;

        let mut conn = T::connect(endpoint).await?;

        let hello_msg = uacp::Hello {
            protocol_version: 0,
            receive_buffer_size: 8192,
            send_buffer_size: 8192,
            max_message_size: 8192 * 16,
            max_chunk_count: 16,
            endpoint_url: endpoint.to_string(),
        };

        conn.send(uacp::Msg::Hello(hello_msg)).await?;

        let msg = conn.recv().await?;
        let uacp::Msg::Acknowledge(ack_msg) = msg else {
            return Err(ua_core::transport::Error::Uacp(
                uacp::ErrorCode::BadTcpMessageTypeInvalid.into(),
            ));
        };

        println!("Acknowledge: {:?}", ack_msg);

        self.transport = Some(conn);
        Ok(())
    }
}
