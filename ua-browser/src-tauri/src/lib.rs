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
            browse, browse_next,
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

fn decode_json<T: open62541::DataType>(json: &str) -> Result<T, String> {
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
        open62541_sys::UA_decodeJson(
            &mut bs as _,
            raw as _,
            T::data_type(),
            &mut options,
        );
    }
    Ok(value)
}

fn encode_json<T: open62541::DataType>(value: &T) -> Result<String, String> {
    let mut options = open62541_sys::UA_EncodeJsonOptions::default();
    options.stringNodeIds = true;
    let size = unsafe {
        open62541_sys::UA_calcSizeJson(value.as_ptr() as _, T::data_type(), &mut options as _)
    };
    if size == 0 {
        return Err("Failed to calculate JSON size".to_string());
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

    let result = String::from_utf8(out_buf).map_err(|e| e.to_string())?;
    Ok(result)
}

// #[tauri::command]
// async fn read_attribute(
//     state: State<'_, Mutex<AppState>>,
//     node_id: ua::NodeId,
//     //attribute_id: uajs::AttributeId,
// ) -> Result<ipc::Response, String> {
// 
//     let state = state.lock().await;
//     let Some(client) = &state.client else {
//         return Err("No client connected".to_string());
//     };
//     let attribute_id: ua::AttributeId = ua::AttributeId::DISPLAYNAME;
//     let value = client
//         .read_attribute(&node_id, &attribute_id)
//         .await
//         .map_err(|e| e.to_string())?;
// 
//     println!("Read attribute value: {:?}", value);
//     let json = if let Some(value) = value.into_value() {
//         println!("Variant value: {:?}", value);
//         encode_json(&value)?
//     } else {
//         "null".to_string()
//     };
//     println!("Resulting JSON: {}", json);
//     let response = ipc::InvokeResponseBody::Json(json);
//     Ok(ipc::Response::new(response))
// }

#[tauri::command]
async fn browse(
    state: State<'_, Mutex<AppState>>,
    req: ipc::Request<'_>,
) -> Result<ipc::Response, String> {
    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };

    let ipc::InvokeBody::Raw(json) = req.body() else {
        return Err("Expected raw JSON body".to_string());
    };
    let json = str::from_utf8(json).map_err(|e| e.to_string())?;
    let req: ua::BrowseRequest = decode_json(json)?;

    let response = client
        .service_request(req)
        .await
        .map_err(|e| e.to_string())?;

    let json = encode_json(&response)?;

    let response = ipc::InvokeResponseBody::Json(json);
    Ok(ipc::Response::new(response))
}

#[tauri::command]
async fn browse_next(
    state: State<'_, Mutex<AppState>>,
    req: ipc::Request<'_>,
) -> Result<ipc::Response, String> {
    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };

    let ipc::InvokeBody::Raw(json) = req.body() else {
        return Err("Expected raw JSON body".to_string());
    };
    let json = str::from_utf8(json).map_err(|e| e.to_string())?;
    let req: ua::BrowseNextRequest = decode_json(json)?;

    let response = client
        .service_request(req)
        .await
        .map_err(|e| e.to_string())?;

    let json = encode_json(&response)?;

    let response = ipc::InvokeResponseBody::Json(json);
    Ok(ipc::Response::new(response))
}
