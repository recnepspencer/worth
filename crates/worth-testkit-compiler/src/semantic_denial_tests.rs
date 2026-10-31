//! Self-contained raw-diagnostic parser tests; these claim no compiler execution.
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::semantic_denial::{check_semantic_outcome, RequiredDiagnostic};
use crate::CompilerFixtureRefusal;

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/semantic_denial_tests.rs")
}

fn outcome() -> Value {
    json!({
        "environment":"certification", "environment_identity":"parser-batch",
        "audience_identity":"parser-audience", "shared_target_root":"parser-target",
        "proof_stage":"check", "fate":"denied", "messages":[{
            "level":"error", "code":{"code":"E0614"},
            "message":"type `PhysicalRecordChunkView<'_>` cannot be dereferenced",
            "rendered":"error[E0614]: PhysicalRecordChunkView cannot be dereferenced",
            "spans":[{"file_name":source(), "label":null}], "children":[]
        }]
    })
}

fn check(observation: &Value) -> Result<(), CompilerFixtureRefusal> {
    check_semantic_outcome(
        &source(),
        observation,
        RequiredDiagnostic::Code("E0614"),
        &["PhysicalRecordChunkView"],
    )
}

#[test]
fn semantic_denial_accepts_harmless_presentation_and_wording_changes() {
    assert!(check(&outcome()).is_ok());
    let mut changed = outcome();
    changed["messages"][0]["rendered"] = "different layout and explanation".into();
    changed["messages"][0]["message"] = "PhysicalRecordChunkView refuses this operation".into();
    assert!(check(&changed).is_ok());
}

#[test]
fn semantic_denial_requires_code_symbols_and_attribution_in_one_error() {
    for mutation in [
        "code",
        "symbol",
        "foreign",
        "split-code",
        "split-attribution",
        "rendered-only",
    ] {
        let mut changed = outcome();
        let mut other = changed["messages"][0].clone();
        match mutation {
            "code" => changed["messages"][0]["code"]["code"] = "E0432".into(),
            "symbol" | "rendered-only" | "split-code" => {
                changed["messages"][0]["message"] = "an unsupported operation on OtherView".into();
            }
            "foreign" | "split-attribution" => {
                changed["messages"][0]["spans"][0]["file_name"] = "foreign.rs".into();
            }
            _ => unreachable!(),
        }
        if mutation.starts_with("split-") {
            other["code"]["code"] = "E0432".into();
            if mutation == "split-attribution" {
                other["message"] = "an unrelated error".into();
            }
            changed["messages"].as_array_mut().unwrap().push(other);
        }
        assert!(
            matches!(
                check(&changed),
                Err(CompilerFixtureRefusal::WrongSemanticDenial { .. })
            ),
            "{mutation}"
        );
    }
}

#[test]
fn semantic_denial_refuses_setup_errors_before_or_after_the_matched_error() {
    for setup_first in [false, true] {
        let mut changed = outcome();
        let mut setup = changed["messages"][0].clone();
        setup["message"] = "no matching package named `missing`".into();
        setup["spans"] = json!([]);
        let messages = changed["messages"].as_array_mut().unwrap();
        messages.insert(if setup_first { 0 } else { 1 }, setup);
        assert!(
            matches!(check(&changed), Err(CompilerFixtureRefusal::WrongSemanticDenial { reason, .. }) if reason.contains("setup failure"))
        );
    }
}

#[test]
fn semantic_denial_refuses_vacuous_expectations() {
    for (code, symbols) in [
        ("", vec!["PhysicalRecordChunkView"]),
        (" ", vec!["PhysicalRecordChunkView"]),
        ("E0614", vec![]),
        ("E0614", vec![""]),
        ("E0614", vec!["PhysicalRecordChunkView", " "]),
    ] {
        assert!(matches!(
            check_semantic_outcome(
                &source(),
                &outcome(),
                RequiredDiagnostic::Code(code),
                &symbols
            ),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
}

#[test]
fn semantic_denial_requires_certification_check_evidence() {
    for stage in ["codegen", "unknown", ""] {
        let mut changed = outcome();
        changed["proof_stage"] = stage.into();
        assert!(matches!(
            check(&changed),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
    let mut changed = outcome();
    changed.as_object_mut().unwrap().remove("proof_stage");
    assert!(matches!(
        check(&changed),
        Err(CompilerFixtureRefusal::InvalidPrebuild(_))
    ));
    for environment in ["production", "unknown"] {
        let mut changed = outcome();
        changed["environment"] = environment.into();
        assert!(matches!(
            check(&changed),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
}

#[test]
fn semantic_denial_preserves_malformed_observation_and_compiled_fate_refusals() {
    for mutation in [
        "audience",
        "batch",
        "target",
        "extra",
        "code",
        "rendered",
        "spans",
        "children",
        "message",
        "span-source",
        "expansion",
    ] {
        let mut changed = outcome();
        match mutation {
            "audience" => changed["audience_identity"] = "".into(),
            "batch" => changed["environment_identity"] = "".into(),
            "target" => changed["shared_target_root"] = "".into(),
            "extra" => changed["undeclared_field"] = true.into(),
            "code" => changed["messages"][0]["code"] = 7.into(),
            "rendered" => changed["messages"][0]["rendered"] = 7.into(),
            "spans" => changed["messages"][0]["spans"] = 7.into(),
            "children" => changed["messages"][0]["children"] = 7.into(),
            "message" => changed["messages"][0]["message"] = 7.into(),
            "span-source" => changed["messages"][0]["spans"][0]["file_name"] = 7.into(),
            "expansion" => changed["messages"][0]["spans"][0]["expansion"] = 7.into(),
            _ => unreachable!(),
        }
        assert!(
            matches!(
                check(&changed),
                Err(CompilerFixtureRefusal::InvalidPrebuild(_))
            ),
            "{mutation}"
        );
    }
    let mut compiled = outcome();
    compiled["fate"] = "compiled".into();
    assert!(matches!(
        check(&compiled),
        Err(CompilerFixtureRefusal::UnexpectedCompilerSuccess(_))
    ));
    let mut no_errors = outcome();
    no_errors["messages"][0]["level"] = "note".into();
    assert!(matches!(
        check(&no_errors),
        Err(CompilerFixtureRefusal::NoCompilerErrors(_))
    ));
}

#[test]
fn semantic_denial_uses_label_child_and_expansion_evidence_without_rendered_source() {
    for location in ["label", "child", "expansion"] {
        let mut changed = outcome();
        let error = &mut changed["messages"][0];
        match location {
            "label" => {
                error["message"] = "unsupported operation".into();
                error["spans"][0]["label"] = "PhysicalRecordChunkView".into();
            }
            "child" => {
                let span = error["spans"].take();
                error["spans"] = json!([]);
                error["children"] = json!([{"level":"note", "message":"usage", "code":null, "rendered":null, "spans":span, "children":[]}]);
            }
            "expansion" => {
                let span = error["spans"][0].clone();
                error["spans"] =
                    json!([{"file_name":"macro.rs", "label":null, "expansion":{"span":span}}]);
            }
            _ => unreachable!(),
        }
        assert!(check(&changed).is_ok(), "{location}");
    }
}

fn uncoded_outcome() -> Value {
    let mut observation = outcome();
    observation["messages"][0]["code"] = Value::Null;
    observation["messages"][0]["message"] =
        "cannot construct OwnerCompletion due to private fields".into();
    observation
}

fn check_uncoded(observation: &Value) -> Result<(), CompilerFixtureRefusal> {
    check_semantic_outcome(
        &source(),
        observation,
        RequiredDiagnostic::ExplicitNullCause("cannot construct"),
        &["OwnerCompletion", "private fields"],
    )
}

#[test]
fn uncoded_semantic_denial_requires_explicit_null_not_missing_or_coded() {
    assert!(check_uncoded(&uncoded_outcome()).is_ok());
    for code in [None, Some(json!({"code":"E0451"})), Some(json!(7))] {
        let mut changed = uncoded_outcome();
        match code {
            Some(code) => changed["messages"][0]["code"] = code,
            None => {
                changed["messages"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("code");
            }
        }
        assert!(check_uncoded(&changed).is_err());
    }
    let mut coded = outcome();
    coded["messages"][0]["code"] = Value::Null;
    assert!(matches!(
        check(&coded),
        Err(CompilerFixtureRefusal::WrongSemanticDenial { .. })
    ));
}

#[test]
fn uncoded_semantic_denial_binds_cause_symbols_and_fixture_in_one_error() {
    for mutation in [
        "cause",
        "symbol",
        "foreign",
        "split-cause",
        "split-attribution",
        "rendered-only",
    ] {
        let mut changed = uncoded_outcome();
        let mut other = changed["messages"][0].clone();
        match mutation {
            "cause" => {
                changed["messages"][0]["message"] = "OwnerCompletion has private fields".into()
            }
            "symbol" | "split-cause" | "rendered-only" => {
                changed["messages"][0]["rendered"] = changed["messages"][0]["message"].clone();
                changed["messages"][0]["message"] =
                    "cannot construct OtherCarrier due to private fields".into();
            }
            "foreign" | "split-attribution" => {
                changed["messages"][0]["spans"][0]["file_name"] = "foreign.rs".into()
            }
            _ => unreachable!(),
        }
        if mutation.starts_with("split-") {
            other["message"] = "unrelated OwnerCompletion with private fields".into();
            changed["messages"].as_array_mut().unwrap().push(other);
        }
        assert!(
            matches!(
                check_uncoded(&changed),
                Err(CompilerFixtureRefusal::WrongSemanticDenial { .. })
            ),
            "{mutation}"
        );
    }
}

#[test]
fn uncoded_semantic_denial_refuses_vacuous_expectations() {
    for (cause, symbols) in [
        ("", vec!["OwnerCompletion"]),
        (" ", vec!["OwnerCompletion"]),
        ("cannot construct", vec![]),
        ("cannot construct", vec![""]),
        ("cannot construct", vec!["OwnerCompletion", " "]),
    ] {
        assert!(matches!(
            check_semantic_outcome(
                &source(),
                &uncoded_outcome(),
                RequiredDiagnostic::ExplicitNullCause(cause),
                &symbols
            ),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
}

#[test]
fn uncoded_semantic_denial_rejects_setup_before_or_after_its_match() {
    for setup_first in [false, true] {
        for field in ["message", "rendered"] {
            let mut changed = uncoded_outcome();
            let mut setup = changed["messages"][0].clone();
            setup[field] = "no matching package named `missing`".into();
            setup["spans"] = json!([]);
            changed["messages"]
                .as_array_mut()
                .unwrap()
                .insert(if setup_first { 0 } else { 1 }, setup);
            assert!(
                matches!(check_uncoded(&changed), Err(CompilerFixtureRefusal::WrongSemanticDenial { reason, .. }) if reason.contains("setup failure"))
            );
        }
    }
}

#[test]
fn uncoded_semantic_denial_preserves_evidence_and_wire_refusals() {
    for (field, value) in [
        ("fate", "compiled"),
        ("environment", "production"),
        ("proof_stage", "codegen"),
        ("audience_identity", ""),
        ("environment_identity", ""),
        ("shared_target_root", ""),
    ] {
        let mut changed = uncoded_outcome();
        changed[field] = value.into();
        assert!(check_uncoded(&changed).is_err(), "{field}");
    }
    for field in ["message", "spans", "children", "rendered"] {
        let mut changed = uncoded_outcome();
        changed["messages"][0][field] = 7.into();
        assert!(
            matches!(
                check_uncoded(&changed),
                Err(CompilerFixtureRefusal::InvalidPrebuild(_))
            ),
            "{field}"
        );
    }
    let mut no_error = uncoded_outcome();
    no_error["messages"][0]["level"] = "note".into();
    assert!(matches!(
        check_uncoded(&no_error),
        Err(CompilerFixtureRefusal::NoCompilerErrors(_))
    ));
}

#[test]
fn uncoded_semantic_denial_accepts_harmless_rendering_changes() {
    let mut changed = uncoded_outcome();
    changed["messages"][0]["rendered"] = "different rendering without identifying fragments".into();
    changed["messages"][0]["message"] =
        "private fields prevent callers: cannot construct OwnerCompletion".into();
    assert!(check_uncoded(&changed).is_ok());
}
