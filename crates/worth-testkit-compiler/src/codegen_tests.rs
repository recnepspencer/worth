//! Synthetic attribution tests; the outer codegen probe supplies real E0080.
use std::path::Path;

use serde_json::json;

use crate::diagnostics::codegen_attributed;

fn error(code: &str) -> serde_json::Value {
    json!({"level":"error", "code":{"code":code}, "spans":[{"file_name":"definition.rs"}]})
}

fn note(source: &str) -> serde_json::Value {
    json!({"level":"note",
        "message":"the above error was encountered while instantiating `fn delegate::<T>`",
        "spans":[{"file_name":source}]})
}

#[test]
fn codegen_error_is_attributed_through_its_exact_instantiation_callsite() {
    assert!(codegen_attributed(
        &[error("E0080"), note("fixture.rs")],
        Path::new("fixture.rs")
    ));
}

#[test]
fn codegen_instantiation_note_must_name_the_selected_fixture() {
    assert!(!codegen_attributed(
        &[error("E0080"), note("another.rs")],
        Path::new("fixture.rs")
    ));
}

#[test]
fn a_note_cannot_attribute_a_previous_or_unrelated_compiler_error() {
    for messages in [
        vec![note("fixture.rs"), error("E0080")],
        vec![error("E0432"), note("fixture.rs")],
        vec![error("E0080"), error("E0080"), note("fixture.rs")],
    ] {
        assert!(!codegen_attributed(&messages, Path::new("fixture.rs")));
    }
}

#[test]
fn every_codegen_error_requires_its_own_attributed_boundary() {
    assert!(codegen_attributed(
        &[
            error("E0080"),
            note("fixture.rs"),
            error("E0080"),
            note("fixture.rs")
        ],
        Path::new("fixture.rs")
    ));
    assert!(!codegen_attributed(
        &[error("E0080"), note("fixture.rs"), error("E0080")],
        Path::new("fixture.rs")
    ));
}

#[test]
fn arbitrary_notes_and_note_only_streams_cannot_establish_a_codegen_denial() {
    let mut unrelated = note("fixture.rs");
    unrelated["message"] = json!("an unrelated note");
    assert!(!codegen_attributed(
        &[error("E0080"), unrelated],
        Path::new("fixture.rs")
    ));
    assert!(!codegen_attributed(
        &[note("fixture.rs")],
        Path::new("fixture.rs")
    ));
}

#[test]
fn check_stage_cannot_use_a_codegen_instantiation_note_as_its_error_span() {
    use crate::denial::{check_outcome, RequiredEnvironment};
    use crate::CompilerFixtureRefusal;

    let mut failure = error("E0080");
    failure["rendered"] = json!("error[E0080]: evaluation failed\n");
    let mut location = note("fixture.rs");
    location["rendered"] = json!("note: instantiating delegate\n");
    let mut outcome = json!({
        "environment":"production", "environment_identity":"declared-batch",
        "audience_identity":"declared-audience", "shared_target_root":"target",
        "proof_stage":"check", "fate":"denied", "messages":[failure,location]
    });
    let expected = "error[E0080]: evaluation failed\nnote: instantiating delegate\n";
    assert!(matches!(
        check_outcome(
            Path::new("fixture.rs"),
            &outcome,
            expected,
            RequiredEnvironment::Any
        ),
        Err(CompilerFixtureRefusal::UnattributedDiagnostics(_))
    ));
    outcome["proof_stage"] = json!("codegen");
    check_outcome(
        Path::new("fixture.rs"),
        &outcome,
        expected,
        RequiredEnvironment::Any,
    )
    .unwrap();
    outcome["fate"] = json!("compiled");
    assert!(matches!(
        check_outcome(
            Path::new("fixture.rs"),
            &outcome,
            expected,
            RequiredEnvironment::Any
        ),
        Err(CompilerFixtureRefusal::UnexpectedCompilerSuccess(_))
    ));
}
