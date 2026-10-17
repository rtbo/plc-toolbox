pub mod uacp {
    //! This module contains the implementation of UA Connection Protocol (UACP)
    use bytes::BufMut;
    use ua_macros::{DecodeBinary, EncodeBinary};
    use crate::msg::binary::{Decode, DecodingError, Encode, check_remaining};
    use crate::ua;
    pub enum Msg {
        Hello(Hello),
        Acknowledge(Acknowledge),
        Error(Error),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Msg {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Msg::Hello(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Hello",
                        &__self_0,
                    )
                }
                Msg::Acknowledge(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Acknowledge",
                        &__self_0,
                    )
                }
                Msg::Error(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Error",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Msg {
        #[inline]
        fn clone(&self) -> Msg {
            match self {
                Msg::Hello(__self_0) => Msg::Hello(::core::clone::Clone::clone(__self_0)),
                Msg::Acknowledge(__self_0) => {
                    Msg::Acknowledge(::core::clone::Clone::clone(__self_0))
                }
                Msg::Error(__self_0) => Msg::Error(::core::clone::Clone::clone(__self_0)),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Msg {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Msg {
        #[inline]
        fn eq(&self, other: &Msg) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (Msg::Hello(__self_0), Msg::Hello(__arg1_0)) => __self_0 == __arg1_0,
                    (Msg::Acknowledge(__self_0), Msg::Acknowledge(__arg1_0)) => {
                        __self_0 == __arg1_0
                    }
                    (Msg::Error(__self_0), Msg::Error(__arg1_0)) => __self_0 == __arg1_0,
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Msg {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<Hello>;
            let _: ::core::cmp::AssertParamIsEq<Acknowledge>;
            let _: ::core::cmp::AssertParamIsEq<Error>;
        }
    }
    impl Encode for Msg {
        fn encoded_len(&self) -> u32 {
            8
                + match self {
                    Msg::Hello(msg) => msg.encoded_len(),
                    Msg::Acknowledge(msg) => msg.encoded_len(),
                    Msg::Error(msg) => msg.encoded_len(),
                }
        }
        fn encode<B: BufMut>(&self, buf: &mut B) {
            match self {
                Msg::Hello(msg) => {
                    buf.put_slice(b"HELF");
                    buf.put_u32_le(8 + msg.encoded_len());
                    msg.encode(buf);
                }
                Msg::Acknowledge(msg) => {
                    buf.put_slice(b"ACKF");
                    buf.put_u32_le(8 + msg.encoded_len());
                    msg.encode(buf);
                }
                Msg::Error(msg) => {
                    buf.put_slice(b"ERRF");
                    buf.put_u32_le(8 + msg.encoded_len());
                    msg.encode(buf);
                }
            }
        }
    }
    impl Decode for Msg {
        fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, DecodingError> {
            let mut msg_type = [0u8; 3];
            check_remaining(buf, 8)?;
            buf.copy_to_slice(&mut msg_type);
            let _reserved = buf.get_u8();
            let message_size = buf.get_u32_le();
            check_remaining(buf, message_size as usize - 8)?;
            match &msg_type {
                b"HEL" => {
                    let hello = Hello::decode(buf)?;
                    Ok(Msg::Hello(hello))
                }
                b"ACK" => {
                    let ack = Acknowledge::decode(buf)?;
                    Ok(Msg::Acknowledge(ack))
                }
                b"ERR" => {
                    let err = Error::decode(buf)?;
                    Ok(Msg::Error(err))
                }
                _ => Err(DecodingError::BadMessageType),
            }
        }
    }
    pub struct Hello {
        pub protocol_version: u32,
        pub receive_buffer_size: u32,
        pub send_buffer_size: u32,
        pub max_message_size: u32,
        pub max_chunk_count: u32,
        pub endpoint_url: String,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Hello {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "protocol_version",
                "receive_buffer_size",
                "send_buffer_size",
                "max_message_size",
                "max_chunk_count",
                "endpoint_url",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.protocol_version,
                &self.receive_buffer_size,
                &self.send_buffer_size,
                &self.max_message_size,
                &self.max_chunk_count,
                &&self.endpoint_url,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "Hello", names, values)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Hello {
        #[inline]
        fn clone(&self) -> Hello {
            Hello {
                protocol_version: ::core::clone::Clone::clone(&self.protocol_version),
                receive_buffer_size: ::core::clone::Clone::clone(
                    &self.receive_buffer_size,
                ),
                send_buffer_size: ::core::clone::Clone::clone(&self.send_buffer_size),
                max_message_size: ::core::clone::Clone::clone(&self.max_message_size),
                max_chunk_count: ::core::clone::Clone::clone(&self.max_chunk_count),
                endpoint_url: ::core::clone::Clone::clone(&self.endpoint_url),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Hello {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Hello {
        #[inline]
        fn eq(&self, other: &Hello) -> bool {
            self.protocol_version == other.protocol_version
                && self.receive_buffer_size == other.receive_buffer_size
                && self.send_buffer_size == other.send_buffer_size
                && self.max_message_size == other.max_message_size
                && self.max_chunk_count == other.max_chunk_count
                && self.endpoint_url == other.endpoint_url
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Hello {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u32>;
            let _: ::core::cmp::AssertParamIsEq<String>;
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Encode for Hello
    where
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        String: ::ua_core::msg::binary::Encode,
    {
        fn encoded_len(&self) -> u32 {
            0 + ::ua_core::msg::binary::Encode::encoded_len(&self.protocol_version)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.receive_buffer_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.send_buffer_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.max_message_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.max_chunk_count)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.endpoint_url)
        }
        fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
            ::ua_core::msg::binary::Encode::encode(&self.protocol_version, buf);
            ::ua_core::msg::binary::Encode::encode(&self.receive_buffer_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.send_buffer_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.max_message_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.max_chunk_count, buf);
            ::ua_core::msg::binary::Encode::encode(&self.endpoint_url, buf);
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Decode for Hello
    where
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        String: ::ua_core::msg::binary::Decode,
    {
        fn decode<B: ::bytes::Buf>(
            buf: &mut B,
        ) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
            Ok(Self {
                protocol_version: ::ua_core::msg::binary::Decode::decode(buf)?,
                receive_buffer_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                send_buffer_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                max_message_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                max_chunk_count: ::ua_core::msg::binary::Decode::decode(buf)?,
                endpoint_url: ::ua_core::msg::binary::Decode::decode(buf)?,
            })
        }
    }
    pub struct Acknowledge {
        pub protocol_version: u32,
        pub receive_buffer_size: u32,
        pub send_buffer_size: u32,
        pub max_message_size: u32,
        pub max_chunk_count: u32,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Acknowledge {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "Acknowledge",
                "protocol_version",
                &self.protocol_version,
                "receive_buffer_size",
                &self.receive_buffer_size,
                "send_buffer_size",
                &self.send_buffer_size,
                "max_message_size",
                &self.max_message_size,
                "max_chunk_count",
                &&self.max_chunk_count,
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for Acknowledge {}
    #[automatically_derived]
    impl ::core::clone::Clone for Acknowledge {
        #[inline]
        fn clone(&self) -> Acknowledge {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Acknowledge {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Acknowledge {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Acknowledge {
        #[inline]
        fn eq(&self, other: &Acknowledge) -> bool {
            self.protocol_version == other.protocol_version
                && self.receive_buffer_size == other.receive_buffer_size
                && self.send_buffer_size == other.send_buffer_size
                && self.max_message_size == other.max_message_size
                && self.max_chunk_count == other.max_chunk_count
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Acknowledge {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Encode for Acknowledge
    where
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
        u32: ::ua_core::msg::binary::Encode,
    {
        fn encoded_len(&self) -> u32 {
            0 + ::ua_core::msg::binary::Encode::encoded_len(&self.protocol_version)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.receive_buffer_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.send_buffer_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.max_message_size)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.max_chunk_count)
        }
        fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
            ::ua_core::msg::binary::Encode::encode(&self.protocol_version, buf);
            ::ua_core::msg::binary::Encode::encode(&self.receive_buffer_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.send_buffer_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.max_message_size, buf);
            ::ua_core::msg::binary::Encode::encode(&self.max_chunk_count, buf);
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Decode for Acknowledge
    where
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
        u32: ::ua_core::msg::binary::Decode,
    {
        fn decode<B: ::bytes::Buf>(
            buf: &mut B,
        ) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
            Ok(Self {
                protocol_version: ::ua_core::msg::binary::Decode::decode(buf)?,
                receive_buffer_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                send_buffer_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                max_message_size: ::ua_core::msg::binary::Decode::decode(buf)?,
                max_chunk_count: ::ua_core::msg::binary::Decode::decode(buf)?,
            })
        }
    }
    #[repr(u32)]
    pub enum ErrorCode {
        /// The server cannot process the request because it is too busy.
        BadTcpServerTooBusy = 0x807d0000,
        /// The type of the message specified in the header invalid.
        BadTcpMessageTypeInvalid = 0x807e0000,
        /// The SecureChannelId and/or TokenId are not currently in use.
        BadTcpSecureChannelUnknown = 0x807f0000,
        /// The size of the message chunk specified in the header is too large.
        BadTcpMessageTooLarge = 0x80800000,
        /// The operation timed out.
        BadTimeout = 0x800a0000,
        /// There are not enough resources to process the request.
        BadTcpNotEnoughResources = 0x80810000,
        /// An internal error occurred.
        BadTcpInternalError = 0x80820000,
        /// The server does not recognize the QueryString specified.
        BadTcpEndpointUrlInvalid = 0x80830000,
        /// An error occurred verifying security.
        BadSecurityChecksFailed = 0x80130000,
        /// The request could not be sent because of a network interruption.
        BadRequestInterrupted = 0x80840000,
        /// Timeout occurred while processing the request.
        BadRequestTimeout = 0x80850000,
        /// The secure channel has been closed.
        BadSecureChannelClosed = 0x80860000,
        /// The token has expired or is not recognized.
        BadSecureChannelTokenUnknown = 0x80870000,
        /// The certificate is not trusted.
        BadCertificateUntrusted = 0x801a0000,
        /// It was not possible to determine if the certificate has been revoked.
        BadCertificateRevocationUnknown = 0x801b0000,
        /// It was not possible to determine if the issuer certificate has been revoked.
        BadCertificateIssuerRevocationUnknown = 0x801c0000,
        /// The certificate has been revoked.
        BadCertificateRevoked = 0x801d0000,
        /// The certificate has expired or is not yet valid.
        BadCertificateTimeInvalid = 0x80140000,
        /// An issuer certificate has expired or is not yet valid.
        BadCertificateIssuerTimeInvalid = 0x80150000,
        /// The certificate may not be used for the requested operation.
        BadCertificateUseNotAllowed = 0x80180000,
        /// The issuer certificate may not be used for the requested operation.
        BadCertificateIssuerUseNotAllowed = 0x80190000,
        /// The issuer certificate has been revoked.
        BadCertificateIssuerRevoked = 0x801e0000,
        /// The sequence number is not valid.
        BadSequenceNumberInvalid = 0x80880000,
        /// The server does not support the requested service.
        BadServiceUnsupported = 0x800b0000,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ErrorCode {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ErrorCode::BadTcpServerTooBusy => "BadTcpServerTooBusy",
                    ErrorCode::BadTcpMessageTypeInvalid => "BadTcpMessageTypeInvalid",
                    ErrorCode::BadTcpSecureChannelUnknown => "BadTcpSecureChannelUnknown",
                    ErrorCode::BadTcpMessageTooLarge => "BadTcpMessageTooLarge",
                    ErrorCode::BadTimeout => "BadTimeout",
                    ErrorCode::BadTcpNotEnoughResources => "BadTcpNotEnoughResources",
                    ErrorCode::BadTcpInternalError => "BadTcpInternalError",
                    ErrorCode::BadTcpEndpointUrlInvalid => "BadTcpEndpointUrlInvalid",
                    ErrorCode::BadSecurityChecksFailed => "BadSecurityChecksFailed",
                    ErrorCode::BadRequestInterrupted => "BadRequestInterrupted",
                    ErrorCode::BadRequestTimeout => "BadRequestTimeout",
                    ErrorCode::BadSecureChannelClosed => "BadSecureChannelClosed",
                    ErrorCode::BadSecureChannelTokenUnknown => {
                        "BadSecureChannelTokenUnknown"
                    }
                    ErrorCode::BadCertificateUntrusted => "BadCertificateUntrusted",
                    ErrorCode::BadCertificateRevocationUnknown => {
                        "BadCertificateRevocationUnknown"
                    }
                    ErrorCode::BadCertificateIssuerRevocationUnknown => {
                        "BadCertificateIssuerRevocationUnknown"
                    }
                    ErrorCode::BadCertificateRevoked => "BadCertificateRevoked",
                    ErrorCode::BadCertificateTimeInvalid => "BadCertificateTimeInvalid",
                    ErrorCode::BadCertificateIssuerTimeInvalid => {
                        "BadCertificateIssuerTimeInvalid"
                    }
                    ErrorCode::BadCertificateUseNotAllowed => {
                        "BadCertificateUseNotAllowed"
                    }
                    ErrorCode::BadCertificateIssuerUseNotAllowed => {
                        "BadCertificateIssuerUseNotAllowed"
                    }
                    ErrorCode::BadCertificateIssuerRevoked => {
                        "BadCertificateIssuerRevoked"
                    }
                    ErrorCode::BadSequenceNumberInvalid => "BadSequenceNumberInvalid",
                    ErrorCode::BadServiceUnsupported => "BadServiceUnsupported",
                },
            )
        }
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for ErrorCode {}
    #[automatically_derived]
    impl ::core::clone::Clone for ErrorCode {
        #[inline]
        fn clone(&self) -> ErrorCode {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ErrorCode {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ErrorCode {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ErrorCode {
        #[inline]
        fn eq(&self, other: &ErrorCode) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ErrorCode {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Encode for ErrorCode {
        fn encoded_len(&self) -> u32 {
            let value: u32 = *self as u32;
            ::ua_core::msg::binary::Encode::encoded_len(&value)
        }
        fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
            let value: u32 = *self as u32;
            ::ua_core::msg::binary::Encode::encode(&value, buf);
        }
    }
    #[automatically_derived]
    impl ErrorCode {
        pub fn from_u32(value: u32) -> Option<Self> {
            match value {
                value if value == Self::BadTcpServerTooBusy as u32 => {
                    Some(Self::BadTcpServerTooBusy)
                }
                value if value == Self::BadTcpMessageTypeInvalid as u32 => {
                    Some(Self::BadTcpMessageTypeInvalid)
                }
                value if value == Self::BadTcpSecureChannelUnknown as u32 => {
                    Some(Self::BadTcpSecureChannelUnknown)
                }
                value if value == Self::BadTcpMessageTooLarge as u32 => {
                    Some(Self::BadTcpMessageTooLarge)
                }
                value if value == Self::BadTimeout as u32 => Some(Self::BadTimeout),
                value if value == Self::BadTcpNotEnoughResources as u32 => {
                    Some(Self::BadTcpNotEnoughResources)
                }
                value if value == Self::BadTcpInternalError as u32 => {
                    Some(Self::BadTcpInternalError)
                }
                value if value == Self::BadTcpEndpointUrlInvalid as u32 => {
                    Some(Self::BadTcpEndpointUrlInvalid)
                }
                value if value == Self::BadSecurityChecksFailed as u32 => {
                    Some(Self::BadSecurityChecksFailed)
                }
                value if value == Self::BadRequestInterrupted as u32 => {
                    Some(Self::BadRequestInterrupted)
                }
                value if value == Self::BadRequestTimeout as u32 => {
                    Some(Self::BadRequestTimeout)
                }
                value if value == Self::BadSecureChannelClosed as u32 => {
                    Some(Self::BadSecureChannelClosed)
                }
                value if value == Self::BadSecureChannelTokenUnknown as u32 => {
                    Some(Self::BadSecureChannelTokenUnknown)
                }
                value if value == Self::BadCertificateUntrusted as u32 => {
                    Some(Self::BadCertificateUntrusted)
                }
                value if value == Self::BadCertificateRevocationUnknown as u32 => {
                    Some(Self::BadCertificateRevocationUnknown)
                }
                value if value == Self::BadCertificateIssuerRevocationUnknown as u32 => {
                    Some(Self::BadCertificateIssuerRevocationUnknown)
                }
                value if value == Self::BadCertificateRevoked as u32 => {
                    Some(Self::BadCertificateRevoked)
                }
                value if value == Self::BadCertificateTimeInvalid as u32 => {
                    Some(Self::BadCertificateTimeInvalid)
                }
                value if value == Self::BadCertificateIssuerTimeInvalid as u32 => {
                    Some(Self::BadCertificateIssuerTimeInvalid)
                }
                value if value == Self::BadCertificateUseNotAllowed as u32 => {
                    Some(Self::BadCertificateUseNotAllowed)
                }
                value if value == Self::BadCertificateIssuerUseNotAllowed as u32 => {
                    Some(Self::BadCertificateIssuerUseNotAllowed)
                }
                value if value == Self::BadCertificateIssuerRevoked as u32 => {
                    Some(Self::BadCertificateIssuerRevoked)
                }
                value if value == Self::BadSequenceNumberInvalid as u32 => {
                    Some(Self::BadSequenceNumberInvalid)
                }
                value if value == Self::BadServiceUnsupported as u32 => {
                    Some(Self::BadServiceUnsupported)
                }
                _ => None,
            }
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Decode for ErrorCode {
        fn decode<B: ::ua_core::msg::binary::Buf>(
            buf: &mut B,
        ) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
            let value = <u32 as ::ua_core::msg::binary::Decode>::decode(buf)?;
            Self::from_u32(value)
                .ok_or_else(|| ::ua_core::msg::binary::DecodingError::InvalidData)
        }
    }
    pub struct Error {
        pub code: ErrorCode,
        pub reason: String,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Error {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "Error",
                "code",
                &self.code,
                "reason",
                &&self.reason,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Error {
        #[inline]
        fn clone(&self) -> Error {
            Error {
                code: ::core::clone::Clone::clone(&self.code),
                reason: ::core::clone::Clone::clone(&self.reason),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Error {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Error {
        #[inline]
        fn eq(&self, other: &Error) -> bool {
            self.code == other.code && self.reason == other.reason
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Error {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<ErrorCode>;
            let _: ::core::cmp::AssertParamIsEq<String>;
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Encode for Error
    where
        ErrorCode: ::ua_core::msg::binary::Encode,
        String: ::ua_core::msg::binary::Encode,
    {
        fn encoded_len(&self) -> u32 {
            0 + ::ua_core::msg::binary::Encode::encoded_len(&self.code)
                + ::ua_core::msg::binary::Encode::encoded_len(&self.reason)
        }
        fn encode<B: ::ua_core::msg::binary::BufMut>(&self, buf: &mut B) {
            ::ua_core::msg::binary::Encode::encode(&self.code, buf);
            ::ua_core::msg::binary::Encode::encode(&self.reason, buf);
        }
    }
    #[automatically_derived]
    impl ::ua_core::msg::binary::Decode for Error
    where
        ErrorCode: ::ua_core::msg::binary::Decode,
        String: ::ua_core::msg::binary::Decode,
    {
        fn decode<B: ::bytes::Buf>(
            buf: &mut B,
        ) -> Result<Self, ::ua_core::msg::binary::DecodingError> {
            Ok(Self {
                code: ::ua_core::msg::binary::Decode::decode(buf)?,
                reason: ::ua_core::msg::binary::Decode::decode(buf)?,
            })
        }
    }
    impl Error {
        pub fn new(code: ErrorCode, reason: String) -> Self {
            Self { code, reason }
        }
    }
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{0:?}, reason: {1}", self.code, self.reason))
        }
    }
    impl std::error::Error for Error {}
    impl From<ErrorCode> for Error {
        fn from(code: ErrorCode) -> Self {
            let status_code = ua::StatusCode::from(code);
            Self {
                code,
                reason: status_code.explanation().to_string(),
            }
        }
    }
}
