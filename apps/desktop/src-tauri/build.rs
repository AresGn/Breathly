include!("src/command_list.rs");

fn main() {
    let built = tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    );
    if let Err(error) = built {
        panic!("construction Tauri : {error}");
    }
}
