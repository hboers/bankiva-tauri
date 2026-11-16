use tauri::Manager;

const APP_OS: &str = if cfg!(target_os = "windows") {
    "windows"
} else if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(target_os = "linux") {
    "linux"
} else if cfg!(target_os = "android") {
    "android"
} else if cfg!(target_os = "ios") {
    "ios"
} else {
    "other"
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut app_url = url::Url::parse(env!("BANKIVA_APP_URL"))
        .expect("BANKIVA_APP_URL must be a valid absolute URL");
    app_url.query_pairs_mut().append_pair("tauri", APP_OS);

    tauri::Builder::default()
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .expect("main window should be configured");
            window.navigate(app_url.clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
