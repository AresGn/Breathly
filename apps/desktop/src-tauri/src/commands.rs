//! Commandes Tauri du nettoyage. Elles ne reçoivent qu'un jeton ou un identifiant (SEC3).

use tauri::State;
use tauri::ipc::Channel;

use crate::error::DesktopError;
use crate::service::CleanupService;
use crate::summary::{CleanupSummary, ProgressEvent};
use crate::worker::ProgressSink;

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri possède les arguments de la commande"
)]
pub fn start_cleanup(
    service: State<'_, CleanupService>,
    token: String,
    on_progress: Channel<ProgressEvent>,
) -> Result<String, DesktopError> {
    service.start(&token, sink_of(on_progress))
}

#[tauri::command(async)]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri possède les arguments de la commande"
)]
pub fn stop_cleanup(
    service: State<'_, CleanupService>,
    task_id: String,
) -> Result<CleanupSummary, DesktopError> {
    service.stop(&task_id)
}

fn sink_of(on_progress: Channel<ProgressEvent>) -> ProgressSink {
    Box::new(move |event| {
        if on_progress.send(event).is_err() {
            tracing::warn!(code = "desktop.progress_lost", "progression non livrée");
        }
    })
}
