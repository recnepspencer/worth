//! Synthetic parser evidence; real compiler sessions remain the court's proof.
use super::*;
use crate::denial::{check_outcome, RequiredEnvironment};
use crate::CompilerFixtureRefusal;
use serde_json::json;
use std::path::{Path, PathBuf};

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/synthetic/sources/fixture_owner/ui/denied.rs")
}
fn summary(text: &str) -> Value {
    json!({"level":"error", "code":null, "spans":[], "children":[], "message":text,
        "rendered": format!("error: {text}\n\n")})
}
fn outcome(messages: Vec<Value>) -> Value {
    json!({"environment":"production", "environment_identity":"synthetic-driver-recipe",
        "audience_identity":"synthetic-audience", "shared_target_root":"target", "proof_stage":"check",
        "fate":"denied", "messages":messages})
}
fn error() -> Value {
    json!({"level":"error", "code":{"code":"E0603"}, "message":"function is private",
        "spans":[{"file_name":source(), "is_primary":true}], "children":[],
        "rendered":format!("error[E0603]: function is private\n --> {}:1:2\n", source().display())})
}
const EXPECTED: &str = "error[E0603]: function is private\n --> tests/ui/denied.rs:1:2\n";

#[test]
fn raw_driver_and_cargo_diagnostics_require_the_same_real_error() {
    for messages in [
        vec![error()],
        vec![error(), summary("aborting due to 1 previous error")],
    ] {
        check_outcome(
            &source(),
            &outcome(messages),
            EXPECTED,
            RequiredEnvironment::Any,
        )
        .unwrap();
    }
    assert!(matches!(
        check_outcome(
            &source(),
            &outcome(vec![summary("aborting due to 1 previous error")]),
            EXPECTED,
            RequiredEnvironment::Any
        ),
        Err(CompilerFixtureRefusal::NoCompilerErrors(_))
    ));
    assert!(crate::diagnostics::codegen_attributed(
        &[error(), summary("aborting due to 1 previous error")],
        &source()
    ));
}

#[test]
fn only_exact_positive_counts_and_complete_driver_grammar_are_summaries() {
    for text in [
        "aborting due to 1 previous error",
        "aborting due to 2 previous errors",
        "aborting due to 12 previous errors; 1 warning emitted",
        "aborting due to 1 previous error; 2 warnings emitted",
    ] {
        assert!(is_abort_summary(&summary(text)), "{text}");
    }
    for text in [
        "aborting due to 0 previous errors",
        "aborting due to 01 previous error",
        "aborting due to 1 previous errors",
        "aborting due to 2 previous error",
        "aborting due to 1 previous error; 0 warnings emitted",
        "aborting due to 1 previous error; another failure",
        "aborting for another reason",
    ] {
        assert!(!is_abort_summary(&summary(text)), "{text}");
    }
}

#[test]
fn codes_spans_children_and_changed_rendering_cannot_hide_real_diagnostics() {
    let original = summary("aborting due to 1 previous error");
    for (field, value) in [
        ("code", json!({"code":"E0603"})),
        ("spans", json!([{"file_name":source(), "is_primary":true}])),
        (
            "children",
            json!([{"level":"note", "message":"another diagnostic"}]),
        ),
        ("rendered", json!("error: a different actual error\n")),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        assert!(!is_abort_summary(&changed));
        assert!(check_outcome(
            &source(),
            &outcome(vec![error(), changed]),
            EXPECTED,
            RequiredEnvironment::Any
        )
        .is_err());
    }
    for field in ["code", "spans", "children"] {
        let mut missing = original.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(!is_abort_summary(&missing));
    }
}
