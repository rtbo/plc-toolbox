use ua_core::ua;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e.name());
        eprintln!("{}", e.explanation());
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

async fn run() -> Result<(), ua::StatusCode> {
    let mut client = ua_client::TcpClient::new();
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
