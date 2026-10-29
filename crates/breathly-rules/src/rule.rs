//! Règle de nettoyage validée : le type ne peut représenter qu'une règle complète et cohérente.

use std::collections::BTreeMap;

use crate::error::RuleDefect;
use crate::path::RulePath;

const RULE_ID_MAX_LEN: usize = 64;

/// Identifiant stable d'une règle : minuscules ASCII, chiffres et tirets isolés (`npm-cache`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(String);

impl RuleId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RuleId {
    type Error = RuleDefect;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let is_word = |part: &str| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        };
        if raw.len() > RULE_ID_MAX_LEN || !raw.split('-').all(is_word) {
            return Err(RuleDefect::InvalidId { value: raw });
        }
        Ok(Self(raw))
    }
}

/// Niveau de risque (GLOSSARY) : 🟢 sans risque, 🟡 à vérifier, 🔴 décision personnelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    Safe,
    Review,
    Personal,
}

impl RiskLevel {
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        [Self::Safe, Self::Review, Self::Personal]
            .into_iter()
            .find(|level| level.key() == key)
    }

    /// Valeur écrite dans le fichier de règles.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Review => "review",
            Self::Personal => "personal",
        }
    }

    /// Un élément 🔴 n'est jamais sélectionnable (AGENTS.md, invariant 4).
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        match self {
            Self::Safe | Self::Review => true,
            Self::Personal => false,
        }
    }
}

/// Système d'exploitation visé par une règle (PRD §0 point 1 : Windows et Linux).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Os {
    Linux,
    Windows,
}

impl Os {
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        [Self::Linux, Self::Windows]
            .into_iter()
            .find(|os| os.key() == key)
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
        }
    }
}

/// Texte d'une règle dans les deux langues exigées (US-024.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedText {
    fr: String,
    en: String,
}

impl LocalizedText {
    pub(crate) const fn new(fr: String, en: String) -> Self {
        Self { fr, en }
    }

    #[must_use]
    pub fn fr(&self) -> &str {
        &self.fr
    }

    #[must_use]
    pub fn en(&self) -> &str {
        &self.en
    }
}

/// Commande officielle de l'outil, **affichée en mode avancé et jamais exécutée** (US-060.4, SEC6).
///
/// Le type n'offre que la lecture du texte : aucune méthode ne lance de programme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformativeCommand(String);

impl InformativeCommand {
    pub(crate) const fn new(text: String) -> Self {
        Self(text)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Chemins d'une règle, par système. Chaque système listé dans `os` a au moins un chemin.
pub type PathsByOs = BTreeMap<Os, Vec<RulePath>>;

/// Ce que la règle désigne : les quatre formes du format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Le contenu d'un dossier entier (un cache).
    Folder { paths: PathsByOs },
    /// Un dossier nommé `folder`, proposé seulement si `marker` existe à côté de lui (dossier de projet recréable),
    /// cherché sous `roots`.
    MarkedFolder {
        folder: String,
        marker: String,
        roots: PathsByOs,
    },
    /// Les fichiers d'un dossier dont l'extension est listée (installeurs de Téléchargements).
    FilesByExtension {
        extensions: Vec<String>,
        folders: PathsByOs,
    },
    /// La corbeille du système : aucun chemin, vidage confirmé seulement (ADR 0005).
    SystemTrash,
}

/// Règle de nettoyage validée (GLOSSARY).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub(crate) id: RuleId,
    pub(crate) name: LocalizedText,
    pub(crate) explain: LocalizedText,
    pub(crate) risk: RiskLevel,
    pub(crate) systems: Vec<Os>,
    pub(crate) target: Target,
    pub(crate) command: Option<InformativeCommand>,
}

impl Rule {
    #[must_use]
    pub const fn id(&self) -> &RuleId {
        &self.id
    }

    #[must_use]
    pub const fn name(&self) -> &LocalizedText {
        &self.name
    }

    #[must_use]
    pub const fn explain(&self) -> &LocalizedText {
        &self.explain
    }

    #[must_use]
    pub const fn risk(&self) -> RiskLevel {
        self.risk
    }

    #[must_use]
    pub fn systems(&self) -> &[Os] {
        &self.systems
    }

    #[must_use]
    pub const fn target(&self) -> &Target {
        &self.target
    }

    #[must_use]
    pub const fn command(&self) -> Option<&InformativeCommand> {
        self.command.as_ref()
    }
}
