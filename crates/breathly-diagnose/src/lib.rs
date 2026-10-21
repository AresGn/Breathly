//! Module `diagnose` : diagnostic « pourquoi mon PC est lent » à partir des mesures de `breathly-platform` (design 0002).
//!
//! Fonctions pures sur des relevés fournis par les ports de `breathly-platform` : aucune lecture du système ici.

mod error;
mod software;

pub use error::DiagnoseError;
pub use software::{
    CoreShare, ProcessDetail, SamplingWindow, SoftwareName, SoftwareUsage, TOP_COUNT, TopSoftware,
    top_software,
};
