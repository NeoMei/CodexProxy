mod commands;
mod codex_config;
mod db;
mod proxy;

use db::Database;
use proxy::{ProxyManager, SharedProxyManager};
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::new().expect("Failed to initialize database");
    let port: u16 = db.get_setting("proxy_port").unwrap_or_else(|_| "15731".to_string())
        .parse().unwrap_or(15731);
    let proxy: SharedProxyManager = Arc::new(ProxyManager::new(port));

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::default(),
            None as Option<Vec<&str>>,
        ))
        .manage(db)
        .manage(proxy.clone())
        .setup(move |app| {
            // Auto-start proxy if setting enabled
            let auto_start = app.state::<Database>()
                .get_setting("auto_start")
                .map(|v| v == "true")
                .unwrap_or(false);

            if auto_start {
                let proxy_state = app.state::<SharedProxyManager>();
                if let Ok(proxy_path) = find_proxy_path(app) {
                    let _ = proxy_state.start(&proxy_path);
                }
            }

            // Tray icon setup
            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
            let _tray = TrayIconBuilder::new()
                .tooltip("Coding Plan Proxy")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_providers,
            commands::save_provider,
            commands::delete_provider,
            commands::generate_id,
            commands::test_connection,
            commands::start_proxy,
            commands::stop_proxy,
            commands::proxy_status,
            commands::proxy_port,
            commands::apply_to_codex,
            commands::read_codex_config,
            commands::set_verified,
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn find_proxy_path(app: &tauri::App) -> Result<String, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let bundled = resource_dir.join("proxy").join("index.mjs");
    if bundled.exists() {
        return Ok(bundled.to_string_lossy().to_string());
    }
    // Dev: relative to project
    let dev = std::env::current_dir().unwrap_or_default()
        .parent().map(|p| p.join("proxy").join("index.mjs"));
    if let Some(p) = dev {
        if p.exists() {
            return Ok(p.to_string_lossy().to_string());
        }
    }
    Err("Proxy index.mjs not found".into())
}
