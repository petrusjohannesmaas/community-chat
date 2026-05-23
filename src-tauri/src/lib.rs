mod mqtt;

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use rumqttc::AsyncClient;

struct MqttState(Arc<Mutex<Option<AsyncClient>>>);
struct UsernameState(Arc<Mutex<String>>);

#[tauri::command]
async fn connect(
    app: tauri::AppHandle,
    username: String,
    mqtt_state: State<'_, MqttState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let client = mqtt::connect(app, username.clone()).await;
    *mqtt_state.0.lock().await = Some(client);
    *username_state.0.lock().await = username;
    Ok(())
}

#[tauri::command]
async fn send_message(
    content: String,
    mqtt_state: State<'_, MqttState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let guard = mqtt_state.0.lock().await;
    let username = username_state.0.lock().await;
    if let Some(client) = guard.as_ref() {
        mqtt::publish(client, &username, &content).await;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MqttState(Arc::new(Mutex::new(None))))
        .manage(UsernameState(Arc::new(Mutex::new(String::new()))))
        .invoke_handler(tauri::generate_handler![connect, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}