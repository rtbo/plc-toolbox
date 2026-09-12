use tauri::{self, Manager, State};
use tauri::async_runtime::Mutex;
use open62541::{AsyncClient};

#[derive(Debug, Default)]
struct AppState {
  client: Option<AsyncClient>,
}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            // Disconnect the client when the AppState is dropped
            tauri::async_runtime::block_on(client.disconnect());
        }
    }
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
    .invoke_handler(tauri::generate_handler![connect, disconnect])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn connect(state: State<'_, Mutex<AppState>>, url: String) -> Result<(), String> {
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
