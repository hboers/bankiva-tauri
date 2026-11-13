use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let env_file = manifest_dir.join("../../../.env");
    println!("cargo:rerun-if-changed={}", env_file.display());

    let env_contents = fs::read_to_string(&env_file)
        .expect("Could not read project .env; APP_URL is required");
    let app_url = env_contents
        .lines()
        .filter_map(|line| line.trim().strip_prefix("APP_URL="))
        .map(|value| {
            value
                .trim()
                .trim_matches(|character| character == '\"' || character == '\'')
        })
        .find(|value| !value.is_empty())
        .expect("APP_URL is required in project .env");

    let parsed_url = url::Url::parse(app_url).expect("APP_URL must be a valid absolute URL");
    assert!(
        matches!(parsed_url.scheme(), "https" | "http") && parsed_url.host_str().is_some(),
        "APP_URL must be an absolute http or https URL"
    );
    println!("cargo:rustc-env=BANKIVA_APP_URL={app_url}");
    tauri_build::build()
}
