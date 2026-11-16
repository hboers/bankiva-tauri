use tauri::{
    WebviewUrl,
    webview::{PageLoadEvent, PermissionKind, PermissionResponse, WebviewWindowBuilder},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_url = url::Url::parse(env!("BANKIVA_APP_URL"))
        .expect("BANKIVA_APP_URL must be a valid absolute URL");

    let app_origin = app_url.origin();

    tauri::Builder::default()
        .setup(move |app| {
            let page_origin = app_origin.clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(app_url.clone()))
                .title("Bankiva")
                .inner_size(1200.0, 800.0)
                .on_permission_request({
                    let app_origin = app_origin.clone();
                    move |webview, kind| {
                        let trusted_origin = webview
                            .url()
                            .map(|url| url.origin() == app_origin)
                            .unwrap_or(false);

                        if trusted_origin && kind == PermissionKind::Camera {
                            PermissionResponse::Allow
                        } else {
                            PermissionResponse::Default
                        }
                    }
                })
                .on_page_load(move |window, payload| {
                    if payload.event() == PageLoadEvent::Finished
                        && payload.url().origin() == page_origin
                    {
                        let _ = window.eval(include_str!("camera_upload.js"));
                    }
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
