// use std::fmt;

// use open62541::{AsyncClient, ua};

// #[derive(Debug)]
// enum Error {
//     ConnectionRejected,
//     OtherOpen62541(open62541::Error),
// }

// impl From<open62541::Error> for Error {
//     fn from(e: open62541::Error) -> Self {
//         if e.status_code() == ua::StatusCode::BADCONNECTIONREJECTED {
//             return Error::ConnectionRejected;
//         }
//         Error::OtherOpen62541(e)
//     }
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Error::ConnectionRejected => write!(f, "Connection rejected"),
//             Error::OtherOpen62541(e) => write!(f, "other Open62541 error: {}", e),
//         }
//     }
// }

#[tokio::main]
async fn main() -> std::process::ExitCode {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        eprintln!("{}", e.explanation());
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

async fn run() -> opcua::status_code::Result<()> {
    let client = opcua::Client::new();
    client.connect("opc.tcp://localhost:4840").await?;

    // let root = "i=84";
    // client.browse(root.to_string()).await?;
    // browse(&client, &root, 0).await?;
    Ok(())
}

// async fn browse(client: &AsyncClient, node_id: &ua::NodeId, indent: usize) -> Result<(), Error> {
//     for _ in 0..indent {
//         print!("  ");
//     }
//     print!("{}", node_id);
//     let display_name = client.read_attribute(node_id, &ua::AttributeId::DISPLAYNAME).await?;
//     if let Some(display_name) = display_name.scalar_value() {
//         let dname = display_name.to_value();
//         if let open62541::VariantValue::Scalar(open62541::ScalarValue::LocalizedText(lt)) = dname {
//             print!("     {}", lt.text());
//         }
//     }
//     println!();

//     let browse_desc = ua::BrowseDescription::default().with_node_id(node_id);
//     let (refs, _) = client.browse(&browse_desc).await?;
//     for r in refs {
//         Box::pin(browse(client, &r.node_id().node_id(), indent + 1)).await?;
//     }
//     Ok(())
// }
