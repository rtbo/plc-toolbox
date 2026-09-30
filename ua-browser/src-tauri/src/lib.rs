// use opcua::ua;
use open62541::ua;
use tauri::async_runtime::Mutex;
use tauri::{self, Manager, State, ipc};

#[derive(Debug, Default)]
struct AppState {
    client: Option<open62541::AsyncClient>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect, disconnect, // read_attribute,
            browse
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn connect(state: State<'_, Mutex<AppState>>, url: String) -> Result<(), String> {
    // let client = opcua::Client::new();
    //client.connect(&url).await.map_err(|e| e.to_string())?;
    let client = open62541::AsyncClient::new(&url).map_err(|e| e.to_string())?;

    let mut state = state.lock().await;
    state.client = Some(client);
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut state = state.lock().await;
    if let Some(client) = state.client.take() {
        let _ = client.disconnect().await;
    }
    Ok(())
}

fn encode_json<T: open62541::DataType>(value: &T, out_buf: &mut Vec<u8>) -> Result<(), String> {
    let mut options = open62541_sys::UA_EncodeJsonOptions::default();
    options.stringNodeIds = true;
    let size = unsafe {
        open62541_sys::UA_calcSizeJson(value.as_ptr() as _, T::data_type(), &mut options as _)
    };
    if size == 0 {
        return Err("Failed to calculate JSON size".to_string());
    }

    let start = out_buf.len();
    out_buf.extend(std::iter::repeat(0u8).take(size));

    let mut bs = open62541_sys::UA_ByteString {
        length: size as _,
        data: out_buf[start..].as_mut_ptr() as _,
    };

    unsafe {
        open62541_sys::UA_encodeJson(
            value.as_ptr() as _,
            T::data_type(),
            &mut bs as _,
            &mut options,
        );
    }

    Ok(())
}

#[tauri::command]
async fn read_attribute(
    state: State<'_, Mutex<AppState>>,
    node_id: ua::NodeId,
    //attribute_id: uajs::AttributeId,
) -> Result<ipc::Response, String> {

    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };
    let attribute_id: ua::AttributeId = ua::AttributeId::DISPLAYNAME;
    let value = client
        .read_attribute(&node_id, &attribute_id)
        .await
        .map_err(|e| e.to_string())?;

    println!("Read attribute value: {:?}", value);
    let json = if let Some(value) = value.into_value() {
        println!("Variant value: {:?}", value);
        let mut out_buf = Vec::new();
        encode_json(&value, &mut out_buf)?;
        String::from_utf8(out_buf).map_err(|e| e.to_string())?
    } else {
        "null".to_string()
    };
    println!("Resulting JSON: {}", json);
    let response = ipc::InvokeResponseBody::Json(json);
    Ok(ipc::Response::new(response))
}

#[tauri::command]
async fn browse(
    state: State<'_, Mutex<AppState>>,
    node_id: Option<ua::NodeId>,
) -> Result<ipc::Response, String> {
    use open62541::DataType;

    //let node_id = node_id.unwrap_or(ua::ns0::ROOTFOLDER);
    // let browse_desc = ua::BrowseDescription::default().with_node_id(node_id);
    // let browse_req = ua::BrowseRequest::default().with_nodes_to_browse(browse_desc.into());

    // let state = state.lock().await;
    // let Some(client) = &state.client else {
    //     return Err("No client connected".to_string());
    // };

    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };

    // let response = client.browse(browse_req).await.map_err(|e| e.to_string())?;
    let node_id = node_id.unwrap_or_else(|| ua::NodeId::ns0(open62541_sys::UA_NS0ID_ROOTFOLDER));
    let browse_desc = ua::BrowseDescription::default()
        .with_node_id(&node_id);

    let browse_req = ua::BrowseRequest::init().with_nodes_to_browse(&[browse_desc]);

    let response = client
        .service_request(browse_req)
        .await
        .map_err(|e| e.to_string())?;

    let mut buf = Vec::new();
    encode_json(&response, &mut buf)?;

    let result = String::from_utf8(buf).map_err(|e| e.to_string())?;
    let response = ipc::InvokeResponseBody::Json(result);
    Ok(ipc::Response::new(response))
    // let (mut children, mut next) = client
    //     .browse(&browse_desc)
    //     .await
    //     .map_err(|e| e.to_string())?;

    // while let Some(ref n) = next {
    //     let n = std::slice::from_ref(n);
    //     let res = client.browse_next(&n).await.map_err(|e| e.to_string())?;
    //     for res in res {
    //         let Ok((more_children, more_next)) = res else {
    //             continue;
    //         };
    //         children.extend(more_children);
    //         next = more_next;
    //     }
    // }

    // let mut out_buf = b"[".to_vec();
    // for (i, child) in children.iter().enumerate() {
    //     if i > 0 {
    //         out_buf.push(b',');
    //     }
    //     encode_json(child, &mut out_buf)?;
    // }
    // out_buf.push(b']');

    // let result = String::from_utf8(out_buf).map_err(|e| e.to_string())?;
    // let response = ipc::InvokeResponseBody::Json(result);
    // Ok(ipc::Response::new(response))
}
