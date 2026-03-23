#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::{
    webview::{NewWindowResponse, WebviewBuilder},
    AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl, WindowEvent,
};

const TOOLBAR_HEIGHT: f64 = 84.0;
const VIEWER_LABEL: &str = "viewer";
/// 默认首页（飞书）
const DEFAULT_HOME: &str = "https://www.feishu.cn";

#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    let parsed = url
        .parse()
        .map_err(|e| format!("invalid url: {e}"))?;

    let viewer = app
        .get_webview(VIEWER_LABEL)
        .ok_or_else(|| "viewer webview not found".to_string())?;

    viewer
        .navigate(parsed)
        .map_err(|e| format!("navigate failed: {e}"))?;

    Ok(())
}

#[tauri::command]
fn navigate_back(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn navigate_forward(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn refresh(app: AppHandle, url: String) -> Result<(), String> {
    let parsed = url
        .parse()
        .map_err(|e| format!("invalid url: {e}"))?;

    let viewer = app
        .get_webview(VIEWER_LABEL)
        .ok_or_else(|| "viewer webview not found".to_string())?;

    viewer
        .navigate(parsed)
        .map_err(|e| format!("refresh failed: {e}"))?;

    Ok(())
}

#[tauri::command]
fn stop(app: AppHandle) -> Result<(), String> {
    let viewer = app
        .get_webview(VIEWER_LABEL)
        .ok_or_else(|| "viewer webview not found".to_string())?;

    viewer
        .navigate("about:blank".parse().map_err(|e| format!("invalid url: {e}"))?)
        .map_err(|e| format!("stop failed: {e}"))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_window("main")
                .ok_or("main window not found")?;

            let size = window.inner_size()?;
            let viewer_height = (size.height as f64 - TOOLBAR_HEIGHT).max(100.0);

            let home: Url = DEFAULT_HOME
                .parse()
                .map_err(|e| format!("invalid DEFAULT_HOME: {e}"))?;

            // 使用 Arc 共享 AppHandle
            let app_handle = Arc::new(app.handle().clone());

            // 为 on_new_window 创建新的 Arc 引用
            let app_handle_for_new_window = Arc::clone(&app_handle);
            let _viewer = window.add_child(
                WebviewBuilder::new(VIEWER_LABEL, WebviewUrl::External(home)).on_new_window(
                    move |url, _features| {
                        if let Some(v) = app_handle_for_new_window.get_webview(VIEWER_LABEL) {
                            let _ = v.navigate(url);
                        }
                        NewWindowResponse::Deny
                    },
                ),
                LogicalPosition::new(0.0, TOOLBAR_HEIGHT),
                LogicalSize::new(size.width as f64, viewer_height),
            )?;

            // 为 on_window_event 创建新的 Arc 引用
            let app_handle_for_resize = Arc::clone(&app_handle);
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(size) = event {
                    let height = (size.height as f64 - TOOLBAR_HEIGHT).max(100.0);
                    if let Some(v) = app_handle_for_resize.get_webview(VIEWER_LABEL) {
                        let _ = v.set_position(LogicalPosition::new(0.0, TOOLBAR_HEIGHT));
                        let _ = v.set_size(LogicalSize::new(size.width as f64, height));
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![open_url, navigate_back, navigate_forward, refresh, stop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
