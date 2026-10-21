//! Port `Clock` : l'heure, pour la rotation de l'historique et les relevés (design 0002 §5).

use std::time::SystemTime;

pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}
