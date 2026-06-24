use crate::db::{Database, Provider};
use crate::proxy::SharedProxyManager;
use crate::codex_config;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn list_providers(db: State<Database>) -> Result<Vec<Provider>, String> {
    db.list_providers()
}

#[tauri::command]
pub fn save_provider(db: State<Database>, provider: Provider) -> Result<(), String> {
    db.upsert_provider(&provider)
}

#[tauri::command]
pub fn delete_provider(db: State<Database>, id: String) -> Result<(), String> {
    db.delete_provider(&id)
}

#[tauri::command]
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
pub async fn test_connection(provider: Provider) -> Result<String, String> {
    codex_config::test_provider_connection(&provider).await
}

#[tauri::command]
pub fn start_proxy(proxy: State<SharedProxyManager>, db: State<Database>) -> Result<(), String> {
    let proxy_path = proxy_path();
    proxy.start(&proxy_path)?;
    // Update all enabled providers to the proxy config
    let providers = db.list_providers()?;
    codex_config::write_model_catalog(&providers)?;
    codex_config::write_codex_auth()?;
    let current_model = db.get_setting("current_model").unwrap_or_default();
    let model = if current_model.is_empty() {
        providers.first().map(|p| p.model.clone()).unwrap_or_default()
    } else { current_model };
    let ctx = providers.iter().find(|p| p.model == model).map(|p| p.context_window).unwrap_or(262144);
    codex_config::write_codex_config(&model, proxy.port(), ctx)?;
    Ok(())
}

#[tauri::command]
pub fn stop_proxy(proxy: State<SharedProxyManager>) -> Result<(), String> {
    proxy.stop()
}

#[tauri::command]
pub fn proxy_status(proxy: State<SharedProxyManager>) -> Result<bool, String> {
    Ok(proxy.is_running())
}

#[tauri::command]
pub fn proxy_port(proxy: State<SharedProxyManager>) -> Result<u16, String> {
    Ok(proxy.port())
}

#[tauri::command]
pub fn apply_to_codex(db: State<Database>, proxy: State<SharedProxyManager>, model: String) -> Result<(), String> {
    let providers = db.list_providers()?;
    let provider = providers.iter().find(|p| p.model == model)
        .ok_or_else(|| format!("Model not found: {}", model))?;
    
    codex_config::write_codex_config(&provider.model, proxy.port(), provider.context_window)?;
    codex_config::write_model_catalog(&providers)?;
    codex_config::write_codex_auth()?;
    db.set_setting("current_model", &model)?;
    Ok(())
}

#[tauri::command]
pub fn read_codex_config() -> Result<String, String> {
    codex_config::read_codex_config()
}

#[tauri::command]
pub fn get_setting(db: State<Database>, key: String) -> Result<String, String> {
    db.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(db: State<Database>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value)
}

fn proxy_path() -> String {
    // In dev: use the proxy directory relative to project root
    // In production: use the bundled resource
    let dev_path = std::env::current_dir()
        .unwrap_or_default()
        .parent()
        .map(|p| p.join("proxy").join("index.mjs"))
        .unwrap_or_default();
    
    if dev_path.exists() {
        return dev_path.to_string_lossy().to_string();
    }

    // Fallback: look next to the executable
    if let Ok(exe) = std::env::current_exe() {
        let bundled = exe.parent().unwrap_or(std::path::Path::new(".")).join("proxy").join("index.mjs");
        if bundled.exists() {
            return bundled.to_string_lossy().to_string();
        }
    }

    "proxy/index.mjs".to_string()
}
