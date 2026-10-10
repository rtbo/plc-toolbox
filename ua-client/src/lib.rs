use ua_core::Connection;
use ua_core::connection::tcp;

pub type TcpClient = Client<tcp::Connection>;

pub struct Client<C> {
    connection: Option<C>,
}

impl<C> Client<C>
where
    C: Connection,
{
    pub fn new() -> Self {
        Client { connection: None }
    }

    pub async fn connect(&mut self, endpoint: &str) -> Result<(), ua_core::connection::Error> {
        use ua_core::msg::uacp;

        let mut conn = C::connect(endpoint).await?;

        let hello_msg = uacp::Hello {
            protocol_version: 0,
            receive_buffer_size: 8192,
            send_buffer_size: 8192,
            max_message_size: 8192 * 16,
            max_chunk_count: 16,
            endpoint_url: endpoint.to_string(),
        };
        println!("endpoint len: {}", endpoint.len());

        conn.send(uacp::Msg(hello_msg)).await?;

        self.connection = Some(conn);
        Ok(())
    }
}
