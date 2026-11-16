use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_url = url::Url::parse(env!("BANKIVA_APP_URL"))
        .expect("BANKIVA_APP_URL must be a valid absolute URL");

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
