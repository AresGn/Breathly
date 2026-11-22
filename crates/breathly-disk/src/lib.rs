//! Modules `disk-scan` et `disk-clean` : analyse du disque, simulation, nettoyage, garde-fous et historique (design 0001 et 0002).
//!
//! Crate créé pour la V0.1 (ADR 0010). Pour l'instant : le module des garde-fous (`guardrails`, ticket #72) ; le
//! reste arrive avec les tickets de la version.

mod guardrails;

pub use guardrails::{Candidate, Guardrails, Refusal, VerifiedItem};
