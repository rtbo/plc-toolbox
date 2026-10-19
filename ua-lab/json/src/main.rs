#![allow(unused)]
use std::slice;
use std::str::FromStr;

use open62541::{DataType, ua};

#[derive(Debug)]
enum Error {
    DecodeJsonError,
    EncodeJsonError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DecodeJsonError => write!(f, "Failed to decode JSON"),
            Error::EncodeJsonError => write!(f, "Failed to encode JSON"),
        }
    }
}

impl std::error::Error for Error {}

fn decode_json<T: open62541::DataType>(json: &str) -> Result<T, Error> {
    let mut options = open62541_sys::UA_DecodeJsonOptions::default();
    let mut bs = open62541_sys::UA_ByteString {
        length: json.len() as _,
        data: json.as_ptr() as _,
    };
    let mut value = T::init();
    // SAFETY: We will not give away the ownership of the raw pointer; it is only used temporarily for the C API call.
    // Also, value is just initialized and does not referenced any allocated memory that would be leaked.
    let raw = unsafe { value.as_mut_ptr() };
    unsafe {
        open62541_sys::UA_decodeJson(&mut bs as _, raw as _, T::data_type(), &mut options);
    }
    Ok(value)
}

fn encode_json<T: open62541::DataType>(value: &T) -> Result<String, Error> {
    let mut options = open62541_sys::UA_EncodeJsonOptions::default();
    options.stringNodeIds = true;
    options.prettyPrint = true;
    let size = unsafe {
        open62541_sys::UA_calcSizeJson(value.as_ptr() as _, T::data_type(), &mut options as _)
    };
    if size == 0 {
        return Err(Error::EncodeJsonError);
    }

    let mut out_buf = Vec::with_capacity(size);
    out_buf.extend(std::iter::repeat(0u8).take(size));

    let mut bs = open62541_sys::UA_ByteString {
        length: size as _,
        data: out_buf[..].as_mut_ptr() as _,
    };

    unsafe {
        open62541_sys::UA_encodeJson(
            value.as_ptr() as _,
            T::data_type(),
            &mut bs as _,
            &mut options,
        );
    }

    let result = String::from_utf8(out_buf).map_err(|_| Error::EncodeJsonError)?;
    Ok(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = open62541::AsyncClient::new("opc.tcp://localhost:4840")?;
    // let node_to_browse = ua::NodeId::from_str("i=84")?;
    // let browse_description = ua::BrowseDescription::default().with_node_id(&node_to_browse);

    let json = tokio::fs::read("browse_request.json").await?;
    let json_str = std::str::from_utf8(&json)?;
    let request: ua::BrowseRequest = decode_json(json_str)?;
    println!("Decoded browse request: {:?}", request);

    // let request =
    //     ua::BrowseRequest::init().with_nodes_to_browse(slice::from_ref(&browse_description));

    // let json_req = encode_json(&request)?;
    // tokio::fs::write("browse_request.json", &json_req).await?;

    let resp = client.service_request(request).await?;
    let json_resp = encode_json(&resp)?;
    tokio::fs::write("browse_response.json", &json_resp).await?;

    // let resp = client.browse(&browse_description).await?;
    // println!("Browse response: {:?}", resp);
    Ok(())
}
