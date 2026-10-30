use super::normalized_source_key;
use super::{
    decode, read_declared_suite, validate_message_shape, validate_observation, CompilerFate,
    PrebuiltCompilerOutcomes, PrebuiltFixtureCompilation,
};
use crate::compiler_boundary::{
    ExpectedCompilerDenial, UiCompilerEnvironment, UiFixtureDeclaration, UiProofSuiteDeclaration,
    UiRunFailure,
};
use std::path::{Path, PathBuf};

#[test]
fn coded_denial_parser_survives_harmless_rendering_and_wording_changes() {
    let (source, observed) = chunk_view_diagnostic_fixture();
    assert!(read_chunk_view(&source, observed.clone()).is_ok());
    let mut changed = observed;
    let error = chunk_view_error(&mut changed);
    error["rendered"] = "a different diagnostic layout and explanation".into();
    error["message"] = "PhysicalRecordChunkView refuses this operation".into();
    assert!(read_chunk_view(&source, changed).is_ok());
}

#[test]
fn coded_denial_parser_rejects_wrong_reasons_and_split_evidence() {
    let (source, observed) = chunk_view_diagnostic_fixture();
    for mutation in [
        "code",
        "symbol",
        "foreign",
        "split-code",
        "split-attribution",
        "rendered-only",
    ] {
        let mut changed = observed.clone();
        let error = chunk_view_error(&mut changed);
        let mut other = error.clone();
        match mutation {
            "code" => error["code"]["code"] = "E0432".into(),
            "symbol" | "rendered-only" | "split-code" => {
                error["message"] = "an unsupported operation on OtherView".into();
                error["children"] = serde_json::json!([]);
                for span in error["spans"].as_array_mut().unwrap() {
                    span["label"] = serde_json::Value::Null;
                }
                if mutation == "rendered-only" {
                    error["rendered"] =
                        "error[E0614]: PhysicalRecordChunkView cannot be dereferenced".into();
                }
            }
            "foreign" | "split-attribution" => {
                error["spans"] = serde_json::json!([{"file_name":"foreign.rs", "label":null}]);
                error["children"] = serde_json::json!([]);
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
                read_chunk_view(&source, changed),
                Err(UiRunFailure::WrongCompilerDenial { .. })
            ),
            "{mutation}"
        );
    }
}

#[test]
fn coded_denial_parser_preserves_setup_refusal_and_compiled_fate() {
    let (source, observed) = chunk_view_diagnostic_fixture();
    let mut changed = observed.clone();
    let mut setup = chunk_view_error(&mut changed).clone();
    setup["message"] = "no matching package named `missing_dependency`".into();
    setup["spans"] = serde_json::json!([]);
    changed["messages"].as_array_mut().unwrap().push(setup);
    assert!(
        matches!(read_chunk_view(&source, changed), Err(UiRunFailure::WrongCompilerDenial { reason, .. }) if reason.contains("forbidden setup"))
    );
    let mut compiled = observed;
    compiled["fate"] = "compiled".into();
    assert!(matches!(
        read_chunk_view(&source, compiled),
        Err(UiRunFailure::UnexpectedCompilerSuccess(_))
    ));
}

#[test]
fn coded_denial_parser_follows_expansion_and_child_attribution() {
    let (source, observed) = chunk_view_diagnostic_fixture();
    for child_attribution in [false, true] {
        let mut changed = observed.clone();
        let error = chunk_view_error(&mut changed);
        let original = error["spans"].clone();
        error["spans"] = serde_json::json!([]);
        if child_attribution {
            error["children"] = serde_json::json!([{"message":"usage", "level":"note", "code":null, "rendered":null, "spans":original, "children":[]}]);
        } else {
            error["spans"] = serde_json::json!([{"file_name":"macro_definition.rs", "label":null, "expansion":{"span":original[0]}}]);
        }
        assert!(read_chunk_view(&source, changed).is_ok());
    }
}

fn chunk_view_diagnostic_fixture() -> (PathBuf, serde_json::Value) {
    // This parser fixture claims no compiler execution or compiled provenance.
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/compiler_boundary/prebuilt_environment/tests.rs")
        .canonicalize()
        .unwrap();
    let observed = serde_json::json!({
        "environment":"certification", "environment_identity":"reader-unit",
        "audience_identity":"reader-unit", "shared_target_root":"reader-unit-target",
        "proof_stage":"check", "fate":"denied", "messages":[{
            "message":"type `PhysicalRecordChunkView<'_>` cannot be dereferenced",
            "level":"error", "code":{"code":"E0614"},
            "rendered":"error[E0614]: PhysicalRecordChunkView cannot be dereferenced",
            "spans":[{"file_name":source, "label":"cannot be dereferenced"}],
            "children":[]
        }]
    });
    (source, observed)
}

fn chunk_view_error(observed: &mut serde_json::Value) -> &mut serde_json::Value {
    observed["messages"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|message| message["level"] == "error" && message["code"]["code"] == "E0614")
        .expect("fixture E0614 error")
}

fn read_chunk_view(
    source: &Path,
    observed: serde_json::Value,
) -> Result<crate::compiler_boundary::UiRunResult, UiRunFailure> {
    let path = Path::new("retained-compiler-outcomes.json");
    let document =
        serde_json::json!({"schema":1,"fixtures":{normalized_source_key(source):observed}});
    let outcomes = decode(&serde_json::to_vec(&document).unwrap(), path)?;
    let declaration = UiProofSuiteDeclaration::new(
        "record-chunk-view-pool-authority",
        UiCompilerEnvironment::Certification,
        vec![UiFixtureDeclaration::new(
            "record_chunk_view_exposes_no_pool_authority",
            source,
            ExpectedCompilerDenial::new(["E0614"], ["PhysicalRecordChunkView"]).unwrap(),
        )
        .unwrap()],
    )
    .unwrap();
    read_declared_suite(source.parent().unwrap(), &declaration, &outcomes, path)
}

#[test]
fn unsupported_or_malformed_outcomes_are_refused_before_fixture_lookup() {
    let path = Path::new("compiler-outcomes.json");
    for bytes in [
        b"not JSON".as_slice(),
        br#"{"schema":2,"fixtures":{}}"#,
        br#"{"schema":1}"#,
        br#"{"schema":1,"fixtures":{"fixture.rs":{}}}"#,
    ] {
        assert!(matches!(
            decode(bytes, path),
            Err(UiRunFailure::MalformedPrebuiltOutcomes { .. })
        ));
    }
}

#[test]
fn fixture_outcomes_require_the_compiled_audience_identity() {
    for bytes in [
        br#"{"schema":1,"fixtures":{"fixture.rs":{"environment":"production","environment_identity":"batch","shared_target_root":"target","proof_stage":"check","fate":"denied","messages":[]}}}"#.as_slice(),
        br#"{"schema":1,"fixtures":{"fixture.rs":{"environment":"production","environment_identity":"batch","audience_identity":"","shared_target_root":"target","proof_stage":"check","fate":"denied","messages":[]}}}"#,
    ] {
        assert!(matches!(
            decode(bytes, Path::new("outcomes.json")),
            Err(UiRunFailure::MalformedPrebuiltOutcomes { .. })
        ));
    }
}

#[test]
fn check_stage_observations_preserve_the_strict_wire_contract() {
    let mut document = check_stage_document();
    let path = Path::new("outcomes.json");
    assert!(decode(&serde_json::to_vec(&document).unwrap(), path).is_ok());
    document["fixtures"]["fixture.rs"]["undeclared_field"] = true.into();
    assert!(matches!(
        decode(&serde_json::to_vec(&document).unwrap(), path),
        Err(UiRunFailure::MalformedPrebuiltOutcomes { .. })
    ));
}

#[test]
fn code_generation_unknown_and_missing_stages_cannot_satisfy_store_check_claims() {
    let path = Path::new("outcomes.json");
    for stage in ["codegen", "unrecognized"] {
        let mut document = check_stage_document();
        document["fixtures"]["fixture.rs"]["proof_stage"] = stage.into();
        let failure = decode(&serde_json::to_vec(&document).unwrap(), path);
        assert!(matches!(
            failure,
            Err(UiRunFailure::MalformedPrebuiltOutcomes { reason, .. })
                if reason.contains("require check-stage evidence")
        ));
    }
    let mut document = check_stage_document();
    document["fixtures"]["fixture.rs"]
        .as_object_mut()
        .unwrap()
        .remove("proof_stage");
    assert!(matches!(
        decode(&serde_json::to_vec(&document).unwrap(), path),
        Err(UiRunFailure::MalformedPrebuiltOutcomes { reason, .. })
            if reason.contains("missing field `proof_stage`")
    ));
}

fn check_stage_document() -> serde_json::Value {
    serde_json::from_str(
        r#"{"schema":1,"fixtures":{"fixture.rs":{"environment":"production","environment_identity":"batch","audience_identity":"audience","shared_target_root":"target","proof_stage":"check","fate":"denied","messages":[]}}}"#,
    ).unwrap()
}

#[test]
fn compilation_in_a_different_environment_cannot_satisfy_a_denial() {
    let fixture = fixture();
    let observed = observation(UiCompilerEnvironment::Certification, CompilerFate::Denied);
    assert!(matches!(
        validate_observation(UiCompilerEnvironment::Production, &fixture, &observed),
        Err(UiRunFailure::WrongCompilerEnvironment {
            expected: UiCompilerEnvironment::Production,
            observed: UiCompilerEnvironment::Certification,
            ..
        })
    ));
}

#[test]
fn compiled_fixtures_are_refused_even_when_another_fixture_failed() {
    let fixture = fixture();
    let observed = observation(UiCompilerEnvironment::Production, CompilerFate::Compiled);
    assert!(matches!(
        validate_observation(UiCompilerEnvironment::Production, &fixture, &observed),
        Err(UiRunFailure::UnexpectedCompilerSuccess(_))
    ));
}

#[test]
fn source_snippets_are_not_raw_compiler_diagnostics() {
    let value = serde_json::from_str(r#"{"rendered":"private field"}"#).unwrap();
    assert!(validate_message_shape(&value).is_err());
}

#[test]
fn a_fixture_missing_from_the_prebuild_is_refused_as_unobserved() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let declaration = UiProofSuiteDeclaration::new(
        "missing-observation",
        UiCompilerEnvironment::Production,
        vec![UiFixtureDeclaration::new(
            "missing",
            root.join("src/compiler_boundary/tests.rs"),
            ExpectedCompilerDenial::semantic_fragments(["private"]).unwrap(),
        )
        .unwrap()],
    )
    .unwrap();
    let outcomes = PrebuiltCompilerOutcomes {
        schema: 1,
        fixtures: std::collections::BTreeMap::new(),
    };
    assert!(matches!(
        read_declared_suite(root, &declaration, &outcomes, Path::new("outcomes.json")),
        Err(UiRunFailure::UnobservedFixture(_))
    ));
}

#[cfg(windows)]
#[test]
fn extended_source_paths_have_the_runners_case_folded_key() {
    assert_eq!(
        normalized_source_key(Path::new(r"\\?\C:\Fixture\Private.rs")),
        "c:/fixture/private.rs"
    );
    assert_eq!(
        normalized_source_key(Path::new(r"\\?\UNC\Host\Share\Private.rs")),
        "//host/share/private.rs"
    );
}

fn fixture() -> UiFixtureDeclaration {
    UiFixtureDeclaration::new(
        "private",
        "private.rs",
        ExpectedCompilerDenial::semantic_fragments(["private"]).unwrap(),
    )
    .unwrap()
}

fn observation(
    environment: UiCompilerEnvironment,
    fate: CompilerFate,
) -> PrebuiltFixtureCompilation {
    PrebuiltFixtureCompilation {
        environment,
        environment_identity: "compiler-batch".to_owned(),
        audience_identity: "compiler-audience".to_owned(),
        shared_target_root: "target".to_owned(),
        proof_stage: "check".to_owned(),
        fate,
        messages: Vec::new(),
    }
}
