use open62541::{ua, AsyncClient};
use tauri::async_runtime::Mutex;
use tauri::{self, Manager, State};

mod uajs;

#[derive(Debug, Default)]
struct AppState {
    client: Option<AsyncClient>,
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
            connect,
            disconnect,
            read_attribute,
            browse
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn connect(state: State<'_, Mutex<AppState>>, url: String) -> Result<(), String> {
    let root_id = ua::NodeId::ns0(open62541_sys::UA_NS0ID_ROOTFOLDER);
    println!("Root ID: {}", root_id);

    let client = AsyncClient::new(&url).map_err(|e| e.to_string())?;
    let mut state = state.lock().await;
    state.client = Some(client);
    Ok(())
}

#[tauri::command]
async fn disconnect(state: State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut state = state.lock().await;
    if let Some(client) = state.client.take() {
        client.disconnect().await;
    }
    Ok(())
}

#[tauri::command]
async fn read_attribute(
    state: State<'_, Mutex<AppState>>,
    node_id: String,
    attribute_id: uajs::AttributeId,
) -> Result<uajs::Variant, String> {
    println!("Reading attribute: node_id={}, attribute_id={:?}", node_id, attribute_id);
    let node_id: ua::NodeId = node_id
        .parse()
        .map_err(|e: open62541::Error| e.to_string())?;
    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };
    let attribute_id: ua::AttributeId = attribute_id.into();
    let value = client
        .read_attribute(&node_id, &attribute_id)
        .await
        .map_err(|e| e.to_string())?;

    let value = value.value();
    if let Some(value) = value {
        Ok(value.into())
    }
    else {
        Err("Attribute not found".to_string())
    }
}

#[tauri::command]
async fn browse(state: State<'_, Mutex<AppState>>, node_id: Option<String>) -> Result<Vec<uajs::ReferenceDescription>, String> {
    let node_id: ua::NodeId = match node_id {
        Some(id) => id.parse().map_err(|e: open62541::Error| e.to_string())?,
        None => ua::NodeId::ns0(open62541_sys::UA_NS0ID_ROOTFOLDER),
    };
    let state = state.lock().await;
    let Some(client) = &state.client else {
        return Err("No client connected".to_string());
    };
    let browse_desc = ua::BrowseDescription::default().with_node_id(&node_id);
    let (children, _) = client
        .browse(&browse_desc)
        .await
        .map_err(|e| e.to_string())?;
    println!("Children: {:?}", children);

    Ok(children.into_iter().map(uajs::ReferenceDescription::from).collect::<Vec<_>>())
}
