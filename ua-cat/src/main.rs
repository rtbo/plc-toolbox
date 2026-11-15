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
    let config = ua_client::Config {
        endpoint: "opc.tcp://localhost:4840".to_string(),
        ..Default::default()
    };
    let client = ua_client::Client::connect(config).await?;

    let cl1 = client.clone();
    let cl2 = client.clone();
    let cl3 = client.clone();

    let srvs = tokio::spawn(async move {
        cl1.send_request(ua::FindServersRequest {
            request_header: ua::RequestHeader {
                timestamp: ua::DateTime::now(),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
    });

    let endpoints = tokio::spawn(async move {
        cl2.send_request(ua::GetEndpointsRequest {
            request_header: ua::RequestHeader {
                timestamp: ua::DateTime::now(),
                ..Default::default()
            },
            endpoint_url: "opc.tcp://localhost:4840".to_string(),
            ..Default::default()
        })
        .await
    });

    let session = tokio::spawn(async move {
        cl3.send_request(ua::CreateSessionRequest {
            request_header: ua::RequestHeader {
                timestamp: ua::DateTime::now(),
                ..Default::default()
            },
            client_description: ua::ApplicationDescription {
                application_uri: "urn:localhost:ua-cat".to_string(),
                product_uri: "urn:localhost:ua-cat".to_string(),
                application_name: ua::LocalizedText {
                    locale: Some(ua::LocaleId("en".to_string())),
                    text: "UA-CAT".to_string(),
                },
                application_type: ua::ApplicationType::Client,
                ..Default::default()
            },
            requested_session_timeout: 60000.0, // 60 seconds
            max_response_message_size: 65536,   // 64 KB
            ..Default::default()
        })
        .await
    });

    let (srvs, endpoints, session) = tokio::join!(srvs, endpoints, session);

    srvs.map_err(|err| {
        eprintln!("FindServers task failed: {err}");
        ua::StatusCode::BadInternalError
    })??;
    endpoints.map_err(|err| {
        eprintln!("GetEndpoints task failed: {err}");
        ua::StatusCode::BadInternalError
    })??;
    session.map_err(|err| {
        eprintln!("CreateSession task failed: {err}");
        ua::StatusCode::BadInternalError
    })??;

    println!("all tasks completed successfully");

    client.close().await?;

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
