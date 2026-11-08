//! Production compiler denial cannot be satisfied by a wider fixture audience.
use crate::semantic_denial::{
    check_semantic_in_environment, CompilerEnvironment, RequiredDiagnostic,
};
use serde_json::json;
use std::path::Path;

#[test]
fn production_denial_requires_its_ordinary_graph_and_attributed_error() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/production_semantic_denial_tests.rs");
    let observed = json!({"environment":"production", "environment_identity":"ordinary-session",
        "audience_identity":"consumer", "shared_target_root":"target", "proof_stage":"check",
        "fate":"denied", "messages":[{"level":"error", "code":{"code":"E0277"},
        "message":"member TopologyContribution cannot be declared in schema ConsumerSchema",
        "spans":[{"file_name":source,"label":null}],"children":[],"rendered":"error"}]});
    let check = |value: &serde_json::Value| {
        check_semantic_in_environment(
            &source,
            value,
            RequiredDiagnostic::Code("E0277"),
            &["TopologyContribution", "ConsumerSchema"],
            CompilerEnvironment::Production,
        )
    };
    assert!(check(&observed).is_ok());
    for (field, value) in [
        ("environment", "certification"),
        ("fate", "compiled"),
        ("proof_stage", "codegen"),
        ("audience_identity", ""),
    ] {
        let mut changed = observed.clone();
        changed[field] = value.into();
        assert!(check(&changed).is_err(), "{field}");
    }
    let mut foreign = observed;
    foreign["messages"][0]["spans"][0]["file_name"] = "foreign.rs".into();
    assert!(check(&foreign).is_err());
}
