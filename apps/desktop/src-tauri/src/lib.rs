//! Application de bureau de Breathly (module `app-ui`, côté Rust).
//!
//! Elle ouvre la fenêtre et expose le cœur à l'interface par des commandes Tauri enregistrées une à une
//! (design 0002 §6). Elle ne contient aucune règle métier : celles-ci vivent dans `crates/`.

mod command_list;
mod commands;
mod error;
mod relay;
mod service;
mod summary;
mod worker;

use commands::{start_cleanup, stop_cleanup};

pub use command_list::COMMANDS;
pub use error::DesktopError;
pub use service::{CleanupPorts, CleanupService};
pub use summary::{CleanupSummary, ProgressEvent, SummaryStatus};
pub use worker::ProgressSink;

/// Lance l'application et bloque jusqu'à la fermeture de la fenêtre.
///
/// # Errors
///
/// Renvoie l'erreur de Tauri si le contexte ou la fenêtre ne peuvent pas être créés.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(CleanupService::system())
        .invoke_handler(tauri::generate_handler![start_cleanup, stop_cleanup])
        .run(tauri::generate_context!())
}
