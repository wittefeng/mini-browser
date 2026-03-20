#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // `add_child` 在 `Window` 上；`WebviewWindow` 只是窗口 + 主 WebView 的封装
            let window = app
                .get_window("main")
                .ok_or("main window not found")?;

            let size = window.inner_size()?;
            let viewer_height = (size.height as f64 - TOOLBAR_HEIGHT).max(100.0);

            let home: Url = DEFAULT_HOME
                .parse()
                .map_err(|e| format!("invalid DEFAULT_HOME: {e}"))?;

            let app_handle = app.handle().clone();
            let viewer = window.add_child(
                WebviewBuilder::new(VIEWER_LABEL, WebviewUrl::External(home)).on_new_window(
                    move |url, _features| {
                        // `target="_blank"` / `window.open` 会走这里：在同一子 WebView 中打开，不弹新窗口
                        let app = app_handle.clone();
                        let app2 = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            if let Some(v) = app2.get_webview(VIEWER_LABEL) {
                                let _ = v.navigate(url);
                            }
                        });
                        NewWindowResponse::Deny
                    },
                ),
                LogicalPosition::new(0.0, TOOLBAR_HEIGHT),
                LogicalSize::new(size.width as f64, viewer_height),
            )?;

            window.on_window_event(move |event| {
                if let WindowEvent::Resized(size) = event {
                    let height = (size.height as f64 - TOOLBAR_HEIGHT).max(100.0);
                    let _ = viewer.set_position(LogicalPosition::new(0.0, TOOLBAR_HEIGHT));
                    let _ = viewer.set_size(LogicalSize::new(size.width as f64, height));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![open_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
