use std::fmt;

use open62541::{AsyncClient, ua};

#[derive(Debug)]
enum Error {
    ConnectionRejected,
    OtherOpen62541(open62541::Error),
}

impl From<open62541::Error> for Error {
    fn from(e: open62541::Error) -> Self {
        if e.status_code() == ua::StatusCode::BADCONNECTIONREJECTED {
            return Error::ConnectionRejected;
        }
        Error::OtherOpen62541(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConnectionRejected => write!(f, "Connection rejected"),
            Error::OtherOpen62541(e) => write!(f, "other Open62541 error: {}", e),
        }
    }
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    if let Err(e) = run().await {
        eprintln!("Error: {:?}", e);
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

async fn run() -> Result<(), Error> {
    let client = AsyncClient::new("opc.tcp://192.168.1.110:4840")?;
    let nid_nsarray = ua::NodeId::ns0(open62541_sys::UA_NS0ID_SERVER_NAMESPACEARRAY);
    let nid_ns = ua::NodeId::ns0(open62541_sys::UA_NS0ID_SERVER_NAMESPACES);
    browse(&client, &nid_nsarray).await?;
    browse(&client, &nid_ns).await?;
    Ok(())
}

async fn browse(client: &AsyncClient, node_id: &ua::NodeId) -> Result<(), Error> {
    let browse_desc = ua::BrowseDescription::default().with_node_id(node_id);
    let (refs, continuation) = client.browse(&browse_desc).await?;
    println!("References: {:?}", refs);
    println!("Continuation: {:?}", continuation);
    Ok(())
}
