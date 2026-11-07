use ua_core::encoding::binary::{self, Decode, Encode};
use ua_core::msg::{self, BinaryEncodingId, uasc};
use ua_core::{SecurityPolicy, SecurityPolicyNone, encoding, transport, ua};

#[derive(Debug)]
pub struct SecureChannel<P: SecurityPolicy = SecurityPolicyNone> {
    uacp: transport::Uacp,
    security_policy: P,
    channel_id: u32,
    token_id: u32,
    sequence_number: u32,
    srv_sequence_number: u32,
}

impl<P: SecurityPolicy> SecureChannel<P> {
    pub async fn open(
        uacp: transport::Uacp,
        security_policy: P,
        request_id: u32,
        msg: ua::OpenSecureChannelRequest,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let mut sc = Self {
            uacp,
            security_policy,
            channel_id: 0,
            token_id: 0,
            sequence_number: 0,
            srv_sequence_number: 0,
        };

        let uri = sc.security_policy.uri();

        sc.send_open(msg, request_id).await?;

        let mut recv_buf = sc.security_policy.recv_msg_chunk(&mut sc.uacp).await?;
        let header = uasc::Header::decode(&mut recv_buf, decoding_ctx)?;
        if header.msg_type != uasc::MsgType::Open || header.is_final != uasc::IsFinal::Final {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let security_header = uasc::AsymmetricSecurityHeader::decode(&mut recv_buf, decoding_ctx)?;
        if security_header.security_policy_uri.as_deref() != Some(uri) {
            return Err(ua::StatusCode::BadSecurityChecksFailed);
        }
        let seq_header = uasc::SequenceHeader::decode(&mut recv_buf, decoding_ctx)?;
        if seq_header.request_id != request_id {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let encoding_node_id = ua::NodeId::decode(&mut recv_buf, decoding_ctx)?;
        if encoding_node_id != ua::OpenSecureChannelResponse::binary_encoding_node_id(0) {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let rsp = ua::OpenSecureChannelResponse::decode(&mut recv_buf, decoding_ctx)?;

        sc.srv_sequence_number = seq_header.sequence_number;
        sc.channel_id = rsp.security_token.channel_id;
        sc.token_id = rsp.security_token.token_id;
        Ok(sc)
    }

    async fn send_open(
        &mut self,
        msg: ua::OpenSecureChannelRequest,
        request_id: u32,
    ) -> Result<(), ua::StatusCode> {
        let encoding_node_id = ua::OpenSecureChannelRequest::binary_encoding_node_id(0);
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        let security_header = self.security_policy.asymmetric_security_header();
        let header = uasc::Header {
            msg_type: uasc::MsgType::Open,
            is_final: uasc::IsFinal::Final,
            msg_size: uasc::Header::LEN
                + security_header.encoded_len()
                + uasc::SequenceHeader::LEN
                + body_len,
            secure_channel_id: self.channel_id,
        };

        let seq_header = uasc::SequenceHeader {
            sequence_number: self.sequence_number,
            request_id,
        };

        self.security_policy
            .send_msg_chunk(&mut self.uacp, header.msg_size, |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encoding_node_id.encode(buf);
                msg.encode(buf);
            })
            .await?;

        self.sequence_number += 1;

        self.uacp.flush().await?;
        Ok(())
    }

    pub async fn send_msg<M>(
        &mut self,
        ns_index: u16,
        msg: M,
        request_id: u32,
    ) -> Result<(), ua::StatusCode>
    where
        M: binary::Encode + msg::BinaryEncodingId,
    {
        // TODO: implement chunking
        let encoding_node_id = M::binary_encoding_node_id(ns_index);
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        let security_header = uasc::SymmetricSecurityHeader {
            token_id: self.token_id,
        };
        let header = uasc::Header {
            msg_type: uasc::MsgType::Msg,
            is_final: uasc::IsFinal::Final,
            msg_size: uasc::Header::LEN
                + uasc::SymmetricSecurityHeader::LEN
                + uasc::SequenceHeader::LEN
                + body_len,
            secure_channel_id: self.channel_id,
        };

        let seq_header = uasc::SequenceHeader {
            sequence_number: self.sequence_number,
            request_id,
        };

        self.security_policy
            .send_msg_chunk(&mut self.uacp, header.msg_size, |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encoding_node_id.encode(buf);
                msg.encode(buf);
            })
            .await?;

        self.sequence_number += 1;

        self.uacp.flush().await?;
        Ok(())
    }

    /// Receives and decodes a message using the namespace table in `decoding_ctx`.
    /// Returns `BadDataTypeIdUnknown` when the encoding ID is not a registered numeric ID.
    pub async fn recv_msg(
        &mut self,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<Box<dyn encoding::Value>, ua::StatusCode> {
        let (encoding_node_id, mut recv_buf) = self.recv_msg_body(decoding_ctx).await?;
        let encoding_id = encoding_node_id
            .numeric_identifier()
            .ok_or(ua::StatusCode::BadDataTypeIdUnknown)?;
        let decode = decoding_ctx
            .get_decoding_fn(encoding_node_id.namespace_index())
            .ok_or(ua::StatusCode::BadDataTypeIdUnknown)?;

        decode(&mut recv_buf, encoding_id, decoding_ctx)?
            .ok_or(ua::StatusCode::BadDataTypeIdUnknown)
    }

    /// Receives a message of the requested type without dynamically dispatching its decoder.
    /// `ns_index` is the namespace index of `T` on the server.
    /// Returns `BadTypeMismatch` when the encoding ID does not match `T`.
    pub async fn recv_msg_typed<T>(
        &mut self,
        ns_index: u16,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<T, ua::StatusCode>
    where
        T: encoding::Value + binary::Decode + msg::BinaryEncodingId,
    {
        let (encoding_node_id, mut recv_buf) = self.recv_msg_body(decoding_ctx).await?;
        if encoding_node_id != T::binary_encoding_node_id(ns_index) {
            return Err(ua::StatusCode::BadTypeMismatch);
        }
        T::decode(&mut recv_buf, decoding_ctx)
    }

    async fn recv_msg_body(
        &mut self,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<(ua::NodeId, &[u8]), ua::StatusCode> {
        let mut recv_buf = self.security_policy.recv_msg_chunk(&mut self.uacp).await?;

        let header = uasc::Header::decode(&mut recv_buf, decoding_ctx)?;
        if header.msg_type != uasc::MsgType::Msg || header.is_final != uasc::IsFinal::Final {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let security_header = uasc::SymmetricSecurityHeader::decode(&mut recv_buf, decoding_ctx)?;
        if security_header.token_id != self.token_id {
            return Err(ua::StatusCode::BadSecureChannelTokenUnknown);
        }

        let seq_header = uasc::SequenceHeader::decode(&mut recv_buf, decoding_ctx)?;
        if seq_header.sequence_number != self.srv_sequence_number + 1 {
            return Err(ua::StatusCode::BadSequenceNumberInvalid);
        }
        self.srv_sequence_number = seq_header.sequence_number;

        let encoding_node_id = ua::NodeId::decode(&mut recv_buf, decoding_ctx)?;
        Ok((encoding_node_id, recv_buf))
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};
    use ua_core::encoding::binary::{self, Encode};
    use ua_core::msg::{BinaryEncodingId, uasc};
    use ua_core::{SecurityPolicyNone, transport, ua};

    use super::SecureChannel;

    async fn pair() -> (SecureChannel, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("opc.tcp://{}", listener.local_addr().unwrap());
        let transport = transport::tcp::Transport::connect(&endpoint, 8192, 8192)
            .await
            .unwrap();
        let (writer, _) = listener.accept().await.unwrap();
        let channel = SecureChannel {
            uacp: transport::Uacp::new(transport),
            security_policy: SecurityPolicyNone,
            channel_id: 1,
            token_id: 2,
            sequence_number: 0,
            srv_sequence_number: 0,
        };
        (channel, writer)
    }

    async fn send(
        writer: &mut TcpStream,
        msg_type: uasc::MsgType,
        token_id: u32,
        sequence_number: u32,
        type_id: ua::NodeId,
        body: &[u8],
    ) {
        let size = uasc::Header::LEN
            + uasc::SymmetricSecurityHeader::LEN
            + uasc::SequenceHeader::LEN
            + type_id.encoded_len()
            + body.len() as u32;
        let mut bytes = vec![0; size as usize];
        let mut buf = &mut bytes[..];
        uasc::Header {
            msg_type,
            is_final: uasc::IsFinal::Final,
            msg_size: size,
            secure_channel_id: 1,
        }
        .encode(&mut buf);
        uasc::SymmetricSecurityHeader { token_id }.encode(&mut buf);
        uasc::SequenceHeader {
            sequence_number,
            request_id: 1,
        }
        .encode(&mut buf);
        type_id.encode(&mut buf);
        buf.copy_from_slice(body);
        writer.write_all(&bytes).await.unwrap();
    }

    fn encoded<T: Encode>(value: &T) -> Vec<u8> {
        let mut bytes = vec![0; value.encoded_len() as usize];
        value.encode(&mut &mut bytes[..]);
        bytes
    }

    #[tokio::test]
    async fn receives_dynamic_and_typed_messages() {
        let (mut channel, mut writer) = pair().await;
        let ctx = binary::DecodingCtx::default();
        let request = ua::ReadRequest {
            max_age: 42.0,
            ..Default::default()
        };
        let body = encoded(&request);
        let id = ua::ReadRequest::binary_encoding_node_id(0);
        send(&mut writer, uasc::MsgType::Msg, 2, 1, id.clone(), &body).await;
        send(&mut writer, uasc::MsgType::Msg, 2, 2, id, &body).await;

        let value: Box<dyn Any> = channel.recv_msg(&ctx).await.unwrap();
        assert_eq!(
            encoded(&*value.downcast::<ua::ReadRequest>().unwrap()),
            body
        );
        assert_eq!(
            encoded(
                &channel
                    .recv_msg_typed::<ua::ReadRequest>(0, &ctx)
                    .await
                    .unwrap()
            ),
            body
        );
    }

    #[tokio::test]
    async fn rejects_type_mismatch_and_keeps_sequence() {
        let (mut channel, mut writer) = pair().await;
        let ctx = binary::DecodingCtx::default();
        let request = ua::ReadRequest::default();
        let id = ua::ReadRequest::binary_encoding_node_id(0);
        let body = encoded(&request);
        send(&mut writer, uasc::MsgType::Msg, 2, 1, id.clone(), &body).await;
        send(&mut writer, uasc::MsgType::Msg, 2, 2, id, &body).await;

        assert!(matches!(
            channel.recv_msg_typed::<ua::WriteRequest>(0, &ctx).await,
            Err(ua::StatusCode::BadTypeMismatch)
        ));
        assert_eq!(
            encoded(
                &channel
                    .recv_msg_typed::<ua::ReadRequest>(0, &ctx)
                    .await
                    .unwrap()
            ),
            body
        );
    }

    #[tokio::test]
    async fn uses_registered_namespace_decoder() {
        let (mut channel, mut writer) = pair().await;
        let mut ctx = binary::DecodingCtx::default();
        ctx.register_ns(3, binary::decode_value::<ua::ReadRequest>);
        let request = ua::ReadRequest::default();
        send(
            &mut writer,
            uasc::MsgType::Msg,
            2,
            1,
            ua::NodeId::Numeric(3, 123),
            &encoded(&request),
        )
        .await;

        let value: Box<dyn Any> = channel.recv_msg(&ctx).await.unwrap();
        assert_eq!(
            encoded(&*value.downcast::<ua::ReadRequest>().unwrap()),
            encoded(&request)
        );
    }

    #[tokio::test]
    async fn typed_receive_decodes_without_namespace_dispatch() {
        fn unexpected_decoder(
            _: &mut binary::DecodeBuf,
            _: u32,
            _: &binary::DecodingCtx,
        ) -> Result<Option<Box<dyn ua_core::encoding::Value>>, ua::StatusCode> {
            panic!("typed receive must not invoke the namespace decoder")
        }

        let (mut channel, mut writer) = pair().await;
        let mut ctx = binary::DecodingCtx::new();
        ctx.register_ns(3, unexpected_decoder);
        let request = ua::ReadRequest {
            max_age: 42.0,
            ..Default::default()
        };
        let body = encoded(&request);
        let id = ua::ReadRequest::binary_encoding_node_id(3);
        send(&mut writer, uasc::MsgType::Msg, 2, 1, id.clone(), &body).await;
        send(&mut writer, uasc::MsgType::Msg, 2, 2, id.clone(), &body).await;
        send(&mut writer, uasc::MsgType::Msg, 2, 3, id, &[]).await;
        send(
            &mut writer,
            uasc::MsgType::Msg,
            2,
            4,
            ua::ReadRequest::binary_encoding_node_id(4),
            &body,
        )
        .await;

        assert!(matches!(
            channel.recv_msg_typed::<ua::ReadRequest>(0, &ctx).await,
            Err(ua::StatusCode::BadTypeMismatch)
        ));
        let decoded = channel
            .recv_msg_typed::<ua::ReadRequest>(3, &ctx)
            .await
            .unwrap();
        assert_eq!(encoded(&decoded), body);
        assert!(matches!(
            channel.recv_msg_typed::<ua::ReadRequest>(3, &ctx).await,
            Err(ua::StatusCode::BadDecodingError)
        ));
        let decoded = channel
            .recv_msg_typed::<ua::ReadRequest>(4, &ctx)
            .await
            .unwrap();
        assert_eq!(encoded(&decoded), body);
    }

    #[tokio::test]
    async fn rejects_unknown_types_and_invalid_bodies() {
        let (mut channel, mut writer) = pair().await;
        let ctx = binary::DecodingCtx::default();
        let ids = [
            ua::NodeId::Numeric(3, 1),
            ua::NodeId::Numeric(0, u32::MAX),
            ua::NodeId::String(0, "unknown".into()),
        ];
        for (index, id) in ids.into_iter().enumerate() {
            send(
                &mut writer,
                uasc::MsgType::Msg,
                2,
                index as u32 + 1,
                id,
                &[],
            )
            .await;
            assert!(matches!(
                channel.recv_msg(&ctx).await,
                Err(ua::StatusCode::BadDataTypeIdUnknown)
            ));
        }
        send(
            &mut writer,
            uasc::MsgType::Msg,
            2,
            4,
            ua::ReadRequest::binary_encoding_node_id(0),
            &[],
        )
        .await;
        assert!(matches!(
            channel.recv_msg(&ctx).await,
            Err(ua::StatusCode::BadDecodingError)
        ));
    }

    #[tokio::test]
    async fn rejects_invalid_message_headers() {
        let (mut channel, mut writer) = pair().await;
        let ctx = binary::DecodingCtx::default();
        let id = ua::ReadRequest::binary_encoding_node_id(0);
        send(&mut writer, uasc::MsgType::Open, 2, 1, id.clone(), &[]).await;
        assert!(matches!(
            channel.recv_msg(&ctx).await,
            Err(ua::StatusCode::BadDecodingError)
        ));
        send(&mut writer, uasc::MsgType::Msg, 3, 1, id.clone(), &[]).await;
        assert!(matches!(
            channel.recv_msg(&ctx).await,
            Err(ua::StatusCode::BadSecureChannelTokenUnknown)
        ));
        send(&mut writer, uasc::MsgType::Msg, 2, 2, id, &[]).await;
        assert!(matches!(
            channel.recv_msg(&ctx).await,
            Err(ua::StatusCode::BadSequenceNumberInvalid)
        ));
    }

    #[tokio::test]
    async fn rejects_truncated_secure_channel_header() {
        let (mut channel, mut writer) = pair().await;
        writer.write_all(b"MSGF\x08\0\0\0").await.unwrap();
        assert!(matches!(
            channel.recv_msg(&binary::DecodingCtx::default()).await,
            Err(ua::StatusCode::BadDecodingError)
        ));
    }
}
