//! Jeton de confirmation lié à un plan, à usage unique et expirant (SEC3, TS-04).

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use breathly_platform::fill_random;

use super::error::CleanError;
use super::plan::{ItemId, PlannedItem, SelectedItem};

/// Durée de validité d'une confirmation. Valeur de départ : le temps d'un écran de confirmation.
pub const CONFIRMATION_TTL: Duration = Duration::from_mins(15);

const USED_LIMIT: usize = 64;

/// Jeton opaque. Son `Debug` ne montre pas la valeur.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ConfirmationToken(String);

impl ConfirmationToken {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Reprend un jeton reçu de l'interface.
    ///
    /// # Errors
    /// `TokenInvalid` si la forme n'est pas celle d'un jeton émis ici.
    pub fn parse(raw: &str) -> Result<Self, CleanError> {
        let hex = raw.len() == 32 && raw.bytes().all(|byte| byte.is_ascii_hexdigit());
        if hex {
            Ok(Self(raw.to_owned()))
        } else {
            Err(CleanError::TokenInvalid)
        }
    }
}

impl fmt::Debug for ConfirmationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ConfirmationToken(..)")
    }
}

/// Ce que `issue` rend : le jeton et les identifiants des éléments figés.
#[derive(Debug)]
pub struct Issued {
    pub confirmation: ConfirmationToken,
    pub ids: Vec<ItemId>,
}

struct Grant {
    plan: Vec<PlannedItem>,
    expires_at: SystemTime,
}

struct State {
    next_item: u64,
    live: HashMap<ConfirmationToken, Grant>,
    used: VecDeque<ConfirmationToken>,
}

/// Conserve les plans confirmables. `issue` est le point que la simulation appellera.
pub struct ConfirmationStore {
    ttl: Duration,
    state: Mutex<State>,
}

impl ConfirmationStore {
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            state: Mutex::new(State {
                next_item: 1,
                live: HashMap::new(),
                used: VecDeque::new(),
            }),
        }
    }

    /// Fige `items` et rend un jeton à usage unique, avec l'identifiant donné à chaque élément (dans l'ordre).
    ///
    /// # Errors
    /// `EmptySelection` si la liste est vide, `EntropyUnavailable` si l'aléa manque, `ClockRange` si
    /// l'échéance ne peut pas être calculée.
    pub fn issue(&self, items: Vec<SelectedItem>, now: SystemTime) -> Result<Issued, CleanError> {
        if items.is_empty() {
            return Err(CleanError::EmptySelection);
        }
        let token = ConfirmationToken(random_token()?);
        let expires_at = now.checked_add(self.ttl).ok_or(CleanError::ClockRange)?;
        let mut state = self.lock();
        let plan = plan_of(&mut state, items);
        let ids = plan.iter().map(|item| item.id().clone()).collect();
        state.live.insert(token.clone(), Grant { plan, expires_at });
        Ok(Issued {
            confirmation: token,
            ids,
        })
    }

    /// Consomme le jeton et rend son plan. Un second appel est refusé.
    ///
    /// # Errors
    /// `TokenInvalid`, `TokenExpired` ou `TokenUsed`. Rien n'est modifié sur le disque.
    pub fn consume(
        &self,
        token: &ConfirmationToken,
        now: SystemTime,
    ) -> Result<Vec<PlannedItem>, CleanError> {
        let mut state = self.lock();
        if state.used.contains(token) {
            return Err(CleanError::TokenUsed);
        }
        let Some(grant) = state.live.remove(token) else {
            return Err(CleanError::TokenInvalid);
        };
        remember_used(&mut state, token.clone());
        if now >= grant.expires_at {
            return Err(CleanError::TokenExpired);
        }
        Ok(grant.plan)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn plan_of(state: &mut State, items: Vec<SelectedItem>) -> Vec<PlannedItem> {
    items
        .into_iter()
        .map(|item| {
            let id = ItemId::issued(state.next_item);
            state.next_item += 1;
            PlannedItem::new(id, item)
        })
        .collect()
}

fn remember_used(state: &mut State, spent: ConfirmationToken) {
    if state.used.len() == USED_LIMIT {
        state.used.pop_front();
    }
    state.used.push_back(spent);
}

fn random_token() -> Result<String, CleanError> {
    let mut bytes = [0_u8; 16];
    fill_random(&mut bytes).map_err(|_| CleanError::EntropyUnavailable)?;
    let mut token = String::with_capacity(32);
    for byte in bytes {
        token.push(hex_digit(byte >> 4));
        token.push(hex_digit(byte & 0x0f));
    }
    Ok(token)
}

fn hex_digit(value: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(value)])
}
