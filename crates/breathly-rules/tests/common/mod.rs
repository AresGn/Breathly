//! Fabrique de fichiers de règles pour les tests : une règle valide, modifiée champ par champ.

use breathly_rules::{RuleDefect, RuleRef, RulesError};

const VALID_FOLDER_RULE: [(&str, &str); 10] = [
    ("id", r#""npm-cache""#),
    ("name.fr", r#""Cache de npm""#),
    ("name.en", r#""npm cache""#),
    (
        "explain.fr",
        r#""Paquets téléchargés par npm, retéléchargés si besoin.""#,
    ),
    (
        "explain.en",
        r#""Packages downloaded by npm, downloaded again when needed.""#,
    ),
    ("risk", r#""safe""#),
    ("os", r#"["linux", "windows"]"#),
    ("kind", r#""folder""#),
    ("paths.linux", r#"["~/.npm/_cacache"]"#),
    ("paths.windows", r#"["%LOCALAPPDATA%/npm-cache/_cacache"]"#),
];

/// Change du TOML de la règle valide : `Some(valeur)` remplace ou ajoute la clé, `None` la retire.
pub type Change<'a> = (&'a str, Option<&'a str>);

pub fn rule_file(changes: &[Change<'_>]) -> String {
    let mut lines: Vec<(String, String)> = VALID_FOLDER_RULE
        .iter()
        .map(|&(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    for &(key, value) in changes {
        lines.retain(|(existing, _)| existing != key);
        if let Some(value) = value {
            lines.push((key.to_owned(), value.to_owned()));
        }
    }
    lines
        .iter()
        .fold(String::from("[[rule]]\n"), |file, (key, value)| {
            file + key + " = " + value + "\n"
        })
}

/// Défaut de la règle refusée, après avoir vérifié que le message la nomme (US-060.2).
pub fn defect_of(result: Result<Vec<breathly_rules::Rule>, RulesError>, id: &str) -> RuleDefect {
    let Err(error) = result else {
        panic!("la règle « {id} » doit être refusée");
    };
    assert!(
        error.to_string().contains(id),
        "le message doit nommer la règle « {id} » : {error}"
    );
    let RulesError::InvalidRule {
        rule: RuleRef::Id(named),
        defect,
    } = error
    else {
        panic!("refus attendu sur la règle {id}, obtenu : {error:?}");
    };
    assert_eq!(named, id);
    defect
}
