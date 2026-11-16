use tauri::{
    WebviewUrl,
    webview::{PageLoadEvent, PermissionKind, PermissionResponse, WebviewWindowBuilder},
};

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
    let app_url = url::Url::parse(env!("BANKIVA_APP_URL"))
        .expect("BANKIVA_APP_URL must be a valid absolute URL");

    let app_origin = app_url.origin();
    let cookie_script = format!(
        "(()=>{{if(location.origin!=={:?})return;document.cookie={:?}+'; Path=/; SameSite=Lax'+(location.protocol==='https:'?'; Secure':'');}})();",
        app_origin.ascii_serialization(),
        format!("bankiva_tauri={APP_OS}")
    );

    tauri::Builder::default()
        .setup(move |app| {
            let page_origin = app_origin.clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(app_url.clone()))
                .title("Bankiva")
                .inner_size(1200.0, 800.0)
                .initialization_script(cookie_script)
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
