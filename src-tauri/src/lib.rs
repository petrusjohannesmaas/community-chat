mod client;

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use client::WsWriter;

struct WsState(Arc<Mutex<Option<WsWriter>>>);
struct UsernameState(Arc<Mutex<String>>);

#[tauri::command]
async fn connect(
    app: tauri::AppHandle,
    username: String,
    ws_state: State<'_, WsState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let writer = client::connect(app, username.clone()).await?;
    *ws_state.0.lock().await = Some(writer);
    *username_state.0.lock().await = username;
    Ok(())
}

#[tauri::command]
async fn send_message(
    content: String,
    ws_state: State<'_, WsState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let mut guard = ws_state.0.lock().await;
    let username = username_state.0.lock().await;
    if let Some(writer) = guard.as_mut() {
        client::send(writer, &username, &content).await;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(WsState(Arc::new(Mutex::new(None))))
        .manage(UsernameState(Arc::new(Mutex::new(String::new()))))
        .invoke_handler(tauri::generate_handler![connect, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
