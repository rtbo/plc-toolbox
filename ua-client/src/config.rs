use std::collections::HashMap;

/// Timeout configuration for the OPC UA client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeout {
    /// The connection timeout duration in milliseconds.
    /// Set to zero to disable the connection timeout.
    /// Defaults to 5000 milliseconds.
    pub connection: u32,

    /// The request timeout duration in milliseconds.
    /// Set to zero to disable the request timeout.
    /// Defaults to 5000 milliseconds.
    pub request: u32,
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            connection: 5000,
            request: 5000,
        }
    }
}

/// Buffer configuration for the OPC UA client.
///
/// It is used for stack profiles based on UA Secure Conversation and Binary data encoding:
///  - UA-TCP UA-SC UA-Binary (URLs starting with `"opc.tcp://"`)
///  - WSS UA-SC UA-Binary (URLs starting with `"wss://"`)
///
/// The buffer configuration is negotiated between the client and the server during the handshake.
/// Therefore those values may be adjusted by the server during the connection process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buffer {
    /// The size of the send buffer.
    /// An initial size is set by the client, but it may be adjusted by the server during the handshake.
    /// The default is 64 KiB.
    pub send_size: usize,
    /// The size of the receive buffer.
    /// An initial size is set by the client, but it may be adjusted by the server during the handshake.
    /// The default is 64 KiB
    pub recv_size: usize,
    /// The maximum size of a request message.
    /// If `None`, there is no limit on the message size. (this is the default)
    ///
    /// This field may be set by the server during the handshake if the server imposes a limit on request message size.
    ///
    /// This field can also be set by the client to impose a lower memory limit than the server for request messages.
    /// In all cases, an error is returned when attempting to encode a request that exceeds the allowed size.
    pub max_request_size: Option<usize>,
    /// The maximum size of a response message.
    /// This will determine the maximum number of chunks a response can be split into.
    /// If `None`, there is no limit on the message size (this is the default).
    ///
    /// This field can be set by the client to impose to the server a memory limit for response messages.
    pub max_response_size: Option<usize>,
}

impl Buffer {
    fn chunk_count(msg_size: usize, chunk_size: usize) -> usize {
        (msg_size + chunk_size - 1) / chunk_size
    }

    pub fn max_send_chunks(&self) -> Option<usize> {
        self.max_request_size
            .map(|size| Buffer::chunk_count(size, self.send_size))
    }

    pub fn max_recv_chunks(&self) -> Option<usize> {
        self.max_response_size
            .map(|size| Buffer::chunk_count(size, self.recv_size))
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            send_size: 65536,
            recv_size: 65536,
            max_request_size: None,
            max_response_size: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// The endpoint URL of the OPC UA server to connect to.
    /// The URL scheme will determine the stack profile to use for the connection
    pub endpoint: String,

    /// The timeout configuration for the client.
    pub timeout: Timeout,

    /// The buffer configuration for the client.
    pub buffer: Buffer,

    /// A map containing external nodeset URIs and the corresponding decoding function
    /// This can be built by the user by using the `ua_nsc` crate in a build script to generate
    /// the necessary decoding functions for external nodeset.
    ///
    /// By default, this map is empty and the client only decodes the NS0 nodeset.
    pub external_ns: HashMap<String, ua_core::encoding::binary::DecodingFn>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_chunks() {
        let tests = &[
            (1024, 256, 4),
            (1025, 256, 5),
            (1024, 512, 2),
            (1023, 512, 2),
            (1024, 1024, 1),
            (1024, 2048, 1),
        ];
        for &(msg_size, chunk_size, expected) in tests {
            assert_eq!(Buffer::chunk_count(msg_size, chunk_size), expected);
        }
    }
}
