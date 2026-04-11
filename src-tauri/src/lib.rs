use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub processes: Vec<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            processes: Vec::new(),
        }
    }
}

fn get_config_path<R: Runtime>(handle: &tauri::AppHandle<R>) -> tauri::Result<PathBuf> {
    handle.path().resolve("config.json", tauri::path::BaseDirectory::AppConfig)
}

fn save_config<R: Runtime>(handle: &tauri::AppHandle<R>, config: &MonitorConfig) -> tauri::Result<()> {
    let path = get_config_path(handle)?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, json);
    }
    Ok(())
}

fn load_config<R: Runtime>(handle: &tauri::AppHandle<R>) -> MonitorConfig {
    if let Ok(path) = get_config_path(handle) {
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str(&json) {
                return config;
            }
        }
    }
    MonitorConfig::default()
}

pub struct CaffeineState {
    pub config: Arc<Mutex<MonitorConfig>>,
    pub process: Arc<Mutex<Option<Child>>>,
    pub is_manual: Arc<Mutex<bool>>,
    pub active_reason: Arc<Mutex<Option<String>>>,
    pub active_monitored_apps: Arc<Mutex<Vec<String>>>,
}

impl Clone for CaffeineState {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            process: Arc::clone(&self.process),
            is_manual: Arc::clone(&self.is_manual),
            active_reason: Arc::clone(&self.active_reason),
            active_monitored_apps: Arc::clone(&self.active_monitored_apps),
        }
    }
}



#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub is_on: bool,
    pub is_manual: bool,
    pub active_reason: Option<String>,
    pub active_processes: Vec<String>,
}

impl Drop for CaffeineState {
    fn drop(&mut self) {
        let mut lock = self.process.lock().unwrap();
        if let Some(mut p) = lock.take() {
            let _ = p.kill();
        }
    }
}

#[tauri::command]
async fn toggle(state: tauri::State<'_, CaffeineState>, app_handle: tauri::AppHandle) -> Result<AppStatus, String> {
    {
        let mut process_lock = state.process.lock().unwrap();
        let mut manual_lock = state.is_manual.lock().unwrap();
        let mut reason_lock = state.active_reason.lock().unwrap();

        if let Some(mut p) = process_lock.take() {
            let _ = p.kill();
            *manual_lock = false;
            *reason_lock = None;
        } else {
            let pid = std::process::id();
            match Command::new("/usr/bin/caffeinate").args(["-d", "-i", "-w", &pid.to_string()]).spawn() {
                Ok(p) => {
                    *process_lock = Some(p);
                    *manual_lock = true;
                    *reason_lock = Some("手動".to_string());
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    } // Drop locks here!
    
    let _ = update_tray_menu(&app_handle, &state);

    let is_on = state.process.lock().unwrap().is_some();
    let is_manual = *state.is_manual.lock().unwrap();
    let active_reason = state.active_reason.lock().unwrap().clone();
    let active_processes = state.active_monitored_apps.lock().unwrap().clone();

    Ok(AppStatus {
        is_on,
        is_manual,
        active_reason,
        active_processes,
    })
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, CaffeineState>) -> Result<AppStatus, String> {
    let is_on = state.process.lock().unwrap().is_some();
    let is_manual = *state.is_manual.lock().unwrap();
    let active_reason = state.active_reason.lock().unwrap().clone();
    let active_processes = state.active_monitored_apps.lock().unwrap().clone();

    Ok(AppStatus {
        is_on,
        is_manual,
        active_reason,
        active_processes,
    })
}

#[tauri::command]
async fn set_procs(state: tauri::State<'_, CaffeineState>, app: tauri::AppHandle, procs: Vec<String>) -> Result<(), String> {
    println!("set_procs command called! procs={:?}", procs);
    let mut c = state.config.lock().unwrap();
    c.processes = procs;
    let _ = save_config(&app, &c);
    Ok(())
}

#[tauri::command]
async fn get_procs(state: tauri::State<'_, CaffeineState>) -> Result<Vec<String>, String> {
    let c = state.config.lock().unwrap();
    Ok(c.processes.clone())
}

#[tauri::command]
async fn get_running_processes() -> Result<Vec<String>, String> {
    let script = "tell application \"System Events\" to get name of every process whose background only is false";
    let output = Command::new("/usr/bin/osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes: Vec<String> = stdout
        .trim()
        .split(", ")
        .map(|s| s.to_string())
        .collect();
    
    processes.sort();
    processes.dedup();
    Ok(processes)
}

async fn check_app_running(app_name: &str) -> bool {
    // Remove .app extension if present for the check
    let clean_name = app_name.strip_suffix(".app").unwrap_or(app_name);
    let script = format!("application \"{}\" is running", clean_name);
    let output = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output();

    if let Ok(o) = output {
        String::from_utf8_lossy(&o.stdout).trim() == "true"
    } else {
        false
    }
}

use tauri::{
    menu::{Menu, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

fn update_tray_menu<R: Runtime>(handle: &tauri::AppHandle<R>, state: &CaffeineState) -> tauri::Result<()> {
    let is_on = state.process.lock().unwrap().is_some();
    let reason = state.active_reason.lock().unwrap().clone();
    
    let status_text = if is_on {
        format!("● スリープ抑制中 ({})", reason.unwrap_or_else(|| "手動".to_string()))
    } else {
        "○ スリープ可能".to_string()
    };

    let menu = Menu::with_id(handle, "tray_menu")?;
    let status_item = tauri::menu::MenuItemBuilder::new(status_text).enabled(false).build(handle)?;
    let separator = PredefinedMenuItem::separator(handle)?;
    let show_item = tauri::menu::MenuItemBuilder::with_id("show", "ウィンドウを表示").enabled(true).build(handle)?;
    let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "終了").enabled(true).build(handle)?;

    menu.append(&status_item)?;
    menu.append(&separator)?;
    menu.append(&show_item)?;
    menu.append(&quit_item)?;

    if let Some(tray) = handle.tray_by_id("main") {
        let icon_name = if is_on { "icons/32x32_active.png" } else { "icons/32x32.png" };
        if let Ok(icon) = tauri::image::Image::from_path(
            handle.path().resolve(icon_name, tauri::path::BaseDirectory::Resource)?
        ) {
            let _ = tray.set_icon(Some(icon));
        }
        tray.set_menu(Some(menu))?;
    }
    
    Ok(())
}

#[tauri::command]
async fn pick_app() -> Result<String, String> {
    println!("pick_app command called!");
    let script = r#"
        tell application (path to frontmost application as text)
            try
                set theFile to (choose file with prompt "監視するアプリケーションを選択してください" of type {"com.apple.application-bundle", "app"})
                tell application "System Events" to return name of theFile
            on error
                return ""
            end try
        end tell
    "#;

    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            return Err("キャンセルされました".to_string());
        }
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.is_empty() {
            return Err("キャンセルされました".to_string());
        }
        Err(err)
    }
}

pub fn run() {
    let state = CaffeineState {
        config: Arc::new(Mutex::new(MonitorConfig::default())),
        process: Arc::new(Mutex::new(None)),
        is_manual: Arc::new(Mutex::new(false)),
        active_reason: Arc::new(Mutex::new(None)),
        active_monitored_apps: Arc::new(Mutex::new(Vec::new())),
    };
    let state_clone = state.clone();
    
    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let handle = app.handle().clone();
            
            // Load persistent config
            {
                let mut c = state_clone.config.lock().unwrap();
                *c = load_config(&handle);
            }
            
            // Setup Tray Icon
            let _ = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |handle, event| {
                    match event.id.0.as_str() {
                        "show" => {
                            if let Some(window) = handle.get_webview_window("main") {
                                let w: tauri::WebviewWindow = window;
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let w: tauri::WebviewWindow = window;
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Initial tray menu
            let _ = update_tray_menu(&handle, &state_clone);

            // Monitoring loop
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    
                    let procs = {
                        let lock = state_clone.config.lock().unwrap();
                        lock.processes.clone()
                    };

                    let mut active_app: Vec<String> = Vec::new();
                    for app in procs {
                        if check_app_running(&app).await {
                            active_app.push(app);
                        }
                    }

                    let changed = {
                        let mut process_lock = state_clone.process.lock().unwrap();
                        let manual_lock = state_clone.is_manual.lock().unwrap();
                        let mut reason_lock = state_clone.active_reason.lock().unwrap();
                        let mut active_mon_lock = state_clone.active_monitored_apps.lock().unwrap();

                        let mut changes_made = false;

                        if !active_app.is_empty() {
                            if *active_mon_lock != active_app {
                                *active_mon_lock = active_app.clone();
                                changes_made = true;
                            }

                            if process_lock.is_none() {
                                let pid = std::process::id();
                                if let Ok(p) = Command::new("/usr/bin/caffeinate").args(["-d", "-i", "-w", &pid.to_string()]).spawn() {
                                    *process_lock = Some(p);
                                    *reason_lock = Some(format!("アプリ監視: {}", active_app.join(", ")));
                                    changes_made = true;
                                }
                            } else if !*manual_lock {
                                let new_reason = Some(format!("アプリ監視: {}", active_app.join(", ")));
                                if *reason_lock != new_reason {
                                    *reason_lock = new_reason;
                                    changes_made = true;
                                }
                            }
                        } else {
                            if !active_mon_lock.is_empty() {
                                active_mon_lock.clear();
                                changes_made = true;
                            }

                            if !*manual_lock && process_lock.is_some() {
                                if let Some(mut child) = process_lock.take() {
                                    let _ = child.kill();
                                    *reason_lock = None;
                                    changes_made = true;
                                }
                            }
                        }
                        
                        changes_made
                    }; // Drop locks here!

                    if changed {
                        let _ = update_tray_menu(&handle, &state_clone);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            toggle,
            get_status,
            set_procs,
            get_procs,
            get_running_processes,
            pick_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
