//! TS-01 : le garde-fou « fichier ouvert » (SEC2, ADR 0020, US-034.1, US-036.4). Les faux de `breathly-platform`
//! décident ce qui est ouvert ; le test d'un fichier réellement ouvert par un autre processus est dans
//! `breathly-platform` (`tests/open_files.rs`), seul crate qui peut lancer un programme.

use std::error::Error;

use breathly_disk::{Candidate, Guardrails, Refusal};
use breathly_platform::ProcessName;

mod common;
use common::{Fixture, TestResult, rule};

fn names(programs: &[&str]) -> Result<Vec<ProcessName>, Box<dyn Error>> {
    Ok(programs
        .iter()
        .map(|name| ProcessName::try_from((*name).to_owned()))
        .collect::<Result<_, _>>()?)
}

#[test]
fn an_open_file_is_refused_with_the_programs_that_hold_it() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/open.log")?;
    fixture.open_files.hold(&item, &["firefox", "code"]);
    let roots = [fixture.home().join(".cache/demo")];
    assert_eq!(
        fixture.verdict(&rule("safe")?, (&roots, &item)),
        Err(Refusal::InUse {
            programs: names(&["code", "firefox"])?
        })
    );
    Ok(())
}

#[test]
fn a_folder_containing_an_open_file_is_refused_whatever_its_risk() -> TestResult {
    let fixture = Fixture::new()?;
    let open = fixture.file(".cache/demo/sub/open.log")?;
    fixture.open_files.hold(&open, &["editeur"]);
    let roots = [fixture.home().join(".cache/demo")];
    let folder = fixture.home().join(".cache/demo/sub");
    for risk in ["safe", "review"] {
        assert_eq!(
            fixture.verdict(&rule(risk)?, (&roots, &folder)),
            Err(Refusal::InUse {
                programs: names(&["editeur"])?
            })
        );
    }
    Ok(())
}

#[test]
fn a_file_next_to_an_open_one_is_still_verified() -> TestResult {
    let fixture = Fixture::new()?;
    let open = fixture.file(".cache/demo/open.log")?;
    let closed = fixture.file(".cache/demo/old.bin")?;
    fixture.open_files.hold(&open, &["editeur"]);
    let roots = [fixture.home().join(".cache/demo")];
    assert_eq!(fixture.verdict(&rule("safe")?, (&roots, &closed)), Ok(()));
    Ok(())
}

#[test]
fn a_detection_that_fails_refuses_the_item_instead_of_allowing_it() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    fixture.open_files.break_detection();
    let roots = [fixture.home().join(".cache/demo")];
    assert_eq!(
        fixture.verdict(&rule("safe")?, (&roots, &item)),
        Err(Refusal::OpenCheckUnavailable)
    );
    Ok(())
}

#[test]
fn an_item_opened_between_the_analysis_and_the_action_is_refused_then_allowed_once_closed()
-> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    let rule = rule("safe")?;
    let roots = [fixture.home().join(".cache/demo")];
    let candidate = Candidate {
        rule: &rule,
        roots: &roots,
        path: &item,
    };
    let guardrails = Guardrails::new(&fixture.paths, &fixture.open_files);
    assert!(guardrails.verify(&candidate).is_ok());
    fixture.open_files.hold(&item, &["editeur"]);
    assert!(matches!(
        guardrails.verify(&candidate),
        Err(Refusal::InUse { .. })
    ));
    fixture.open_files.release_all();
    assert!(guardrails.verify(&candidate).is_ok());
    Ok(())
}

#[test]
fn the_refusals_have_a_stable_code_and_a_translation_key() -> TestResult {
    let in_use = Refusal::InUse {
        programs: names(&["editeur"])?,
    };
    assert_eq!(in_use.code(), "guard.in_use");
    assert_eq!(in_use.translation_key(), "errors.guard.in_use");
    assert_eq!(
        Refusal::OpenCheckUnavailable.code(),
        "guard.open_check_unavailable"
    );
    assert_eq!(
        Refusal::OpenCheckUnavailable.translation_key(),
        "errors.guard.open_check_unavailable"
    );
    Ok(())
}
