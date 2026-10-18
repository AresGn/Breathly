//! Application de bureau de Breathly (module `app-ui`, côté Rust).
//!
//! Elle ouvre la fenêtre et expose le cœur à l'interface par des commandes Tauri enregistrées une à une
//! (design 0002 §6). Elle ne contient aucune règle métier : celles-ci vivent dans `crates/`.

/// Lance l'application et bloque jusqu'à la fermeture de la fenêtre.
///
/// # Errors
///
/// Renvoie l'erreur de Tauri si le contexte ou la fenêtre ne peuvent pas être créés.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default().run(tauri::generate_context!())
}
