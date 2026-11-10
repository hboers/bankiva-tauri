pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_url = url::Url::parse(env!("BANKIVA_APP_URL"))?;
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(app_url))
                .title("Bankiva")
                .inner_size(1200.0, 800.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
