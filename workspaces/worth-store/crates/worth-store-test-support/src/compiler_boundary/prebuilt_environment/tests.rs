#[cfg(windows)]
use super::normalized_source_key;
use super::{
    decode, read_declared_suite, validate_message_shape, validate_observation, CompilerFate,
    PrebuiltCompilerOutcomes, PrebuiltFixtureCompilation,
};
use crate::compiler_boundary::{
    ExpectedCompilerDenial, UiCompilerEnvironment, UiFixtureDeclaration, UiProofSuiteDeclaration,
    UiRunFailure,
};
use std::path::Path;

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
        shared_target_root: "target".to_owned(),
        fate,
        messages: Vec::new(),
    }
}
