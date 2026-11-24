//! Simulation : ce que ferait le nettoyage, sans rien modifier ni écrire dans l'historique (US-032, TS-04).
//!
//! Chaque élément passe par les mêmes garde-fous et la même règle de destination que le nettoyage ; seule l'action
//! manque. La simulation délivre le jeton à usage unique que `start_cleanup` exigera (SEC3).

use breathly_platform::{ByteSize, MeasureError};
use std::time::SystemTime;

use super::destination::Destination;
use super::error::CleanError;
use super::plan::{ItemId, SelectedItem};
use super::token::{ConfirmationStore, ConfirmationToken};
use crate::{Candidate, Guardrails, Refusal};

/// Pourquoi un élément n'est pas traité : un code stable et une clé de traduction, sans chemin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reason {
    pub code: &'static str,
    pub translation_key: &'static str,
}

/// Ce que le nettoyage ferait d'un élément.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foreseen {
    Act {
        destination: Destination,
        size: ByteSize,
    },
    /// Refusé par un garde-fou ou laissé en place : expliqué à l'utilisateur (US-036.4).
    Skip(Reason),
    /// La taille n'a pas pu être lue : le nettoyage échouerait sur cet élément.
    Fail(Reason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatedItem {
    pub id: ItemId,
    pub foreseen: Foreseen,
}

/// Le rapport de la simulation : les deux chiffres que le résumé final reprendra (ADR 0005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationReport {
    pub items: Vec<SimulatedItem>,
    /// Octets qui iraient à la corbeille.
    pub trashed: ByteSize,
    /// Octets qui seraient libérés maintenant (suppression définitive des caches 🟢).
    pub freed_now: ByteSize,
}

#[derive(Debug)]
pub struct Simulation {
    pub confirmation: ConfirmationToken,
    pub report: SimulationReport,
}

/// Les éléments d'une simulation et l'heure à laquelle elle est faite.
pub struct SimulationRequest<'a> {
    pub items: Vec<SelectedItem>,
    pub guardrails: &'a Guardrails<'a>,
    pub store: &'a ConfirmationStore,
    pub now: SystemTime,
}

/// Simule `request.items` sans rien modifier, puis fige la sélection derrière un jeton.
///
/// Le plan figé garde tous les éléments, y compris ceux que la simulation ignore : l'exécuteur (#80) revérifie
/// chaque élément au moment d'agir (US-036.4), la simulation n'est qu'un aperçu.
///
/// # Errors
/// [`CleanError`] si la sélection est vide, si l'aléa ou l'horloge manquent. Un élément refusé n'est pas une
/// erreur : il figure dans le rapport comme ignoré.
pub fn simulate(request: SimulationRequest<'_>) -> Result<Simulation, CleanError> {
    let foreseen: Vec<Foreseen> = request
        .items
        .iter()
        .map(|item| foresee(request.guardrails, item))
        .collect();
    let issued = request.store.issue(request.items, request.now)?;
    Ok(Simulation {
        confirmation: issued.confirmation,
        report: report_of(issued.ids, foreseen),
    })
}

/// Ce que le nettoyage fera de `item` : la seule lecture que simulation et exécuteur partagent.
#[must_use]
pub fn foresee(guardrails: &Guardrails<'_>, item: &SelectedItem) -> Foreseen {
    let verified = match guardrails.verify(&Candidate {
        rule: &item.rule,
        roots: &item.roots,
        path: &item.path,
    }) {
        Ok(verified) => verified,
        Err(refusal) => return Foreseen::Skip(reason_of_refusal(&refusal)),
    };
    let Some(destination) = Destination::of(verified.risk()) else {
        return Foreseen::Skip(reason_of_refusal(&Refusal::NotSelectable));
    };
    match verified.anchor().byte_size() {
        Ok(size) => Foreseen::Act { destination, size },
        Err(MeasureError::CrossVolume) => {
            Foreseen::Skip(reason_of_measure(MeasureError::CrossVolume))
        }
        Err(error) => Foreseen::Fail(reason_of_measure(error)),
    }
}

fn report_of(ids: Vec<ItemId>, foreseen: Vec<Foreseen>) -> SimulationReport {
    let mut report = SimulationReport {
        items: Vec::with_capacity(ids.len()),
        trashed: ByteSize::from_bytes(0),
        freed_now: ByteSize::from_bytes(0),
    };
    for (id, foreseen) in ids.into_iter().zip(foreseen) {
        if let Foreseen::Act { destination, size } = foreseen {
            match destination {
                Destination::Trash => report.trashed = report.trashed.saturating_add(size),
                Destination::Permanent => report.freed_now = report.freed_now.saturating_add(size),
            }
        }
        report.items.push(SimulatedItem { id, foreseen });
    }
    report
}

fn reason_of_refusal(refusal: &Refusal) -> Reason {
    Reason {
        code: refusal.code(),
        translation_key: refusal.translation_key(),
    }
}

fn reason_of_measure(error: MeasureError) -> Reason {
    Reason {
        code: error.code(),
        translation_key: error.translation_key(),
    }
}
