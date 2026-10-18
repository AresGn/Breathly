// Évite la fenêtre de console supplémentaire sous Windows en version de production : ne pas retirer.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> tauri::Result<()> {
    breathly_desktop_lib::run()
}
