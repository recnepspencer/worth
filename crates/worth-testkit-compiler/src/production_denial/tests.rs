//! Value-level parser and selection proofs; no compiler execution is claimed.
use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/production_denial/tests.rs")
}

fn observation(symbols: &[&str]) -> Value {
    json!({
        "environment":"production", "environment_identity":"parser-batch",
        "audience_identity":"parser-audience", "shared_target_root":"parser-target",
        "proof_stage":"check", "fate":"denied",
        "messages":symbols.iter().map(|symbol| json!({
            "level":"error", "code":{"code":"E0624"},
            "message":format!("associated function {symbol} is private"),
            "rendered":"presentation is not the fence",
            "spans":[{"file_name":source(), "label":null}], "children":[]
        })).collect::<Vec<_>>()
    })
}

fn coded<'a>(symbols: &'a [&'a str]) -> ProductionDiagnosticExpectation<'a> {
    ProductionDiagnosticExpectation::Code {
        code: "E0624",
        symbols,
    }
}

#[test]
fn production_denial_requires_each_declared_error_once_without_extra_errors() {
    let requirements = [coded(&["Owner"]), coded(&["Owner"])];
    assert!(
        check_production_outcome(&source(), &observation(&["Owner", "Owner"]), &requirements)
            .is_ok()
    );
    for symbols in [
        vec!["Owner"],
        vec!["Owner", "Other"],
        vec!["Owner", "Owner", "Owner"],
    ] {
        assert!(matches!(
            check_production_outcome(&source(), &observation(&symbols), &requirements),
            Err(CompilerFixtureRefusal::WrongSemanticDenial { .. })
        ));
    }
}

#[test]
fn overlapping_error_obligations_find_a_distinct_complete_assignment() {
    let requirements = [coded(&["Owner"]), coded(&["OwnerFirst"])];
    assert!(check_production_outcome(
        &source(),
        &observation(&["OwnerFirst", "OwnerSecond"]),
        &requirements
    )
    .is_ok());
}

#[test]
fn production_semantic_denial_rejects_wrong_graph_stage_and_setup() {
    let requirements = [coded(&["Owner"])];
    for (field, value) in [
        ("environment", "certification"),
        ("proof_stage", "codegen"),
        ("fate", "compiled"),
    ] {
        let mut changed = observation(&["Owner"]);
        changed[field] = value.into();
        assert!(check_production_outcome(&source(), &changed, &requirements).is_err());
    }
    let mut changed = observation(&["Owner", "no matching package"]);
    assert!(
        matches!(check_production_outcome(&source(), &changed, &requirements), Err(CompilerFixtureRefusal::WrongSemanticDenial { reason, .. }) if reason.contains("setup failure"))
    );
    changed["messages"][1]["message"] = "Owner".into();
    changed["messages"][1]["spans"] = json!([]);
    assert!(
        check_production_outcome(&source(), &changed, &[coded(&["Owner"]), coded(&["Owner"])])
            .is_err()
    );
}

#[test]
fn production_semantic_denial_preserves_explicit_uncoded_cause_and_attribution() {
    let requirements = [ProductionDiagnosticExpectation::Uncoded {
        cause: "cannot construct",
        symbols: &["Owner", "private fields"],
    }];
    let mut changed = observation(&["Owner"]);
    changed["messages"][0]["code"] = Value::Null;
    changed["messages"][0]["message"] = "cannot construct Owner due to private fields".into();
    assert!(check_production_outcome(&source(), &changed, &requirements).is_ok());
    for mutation in ["code", "attribution", "cause", "rendered-only"] {
        let mut invalid = changed.clone();
        match mutation {
            "code" => {
                invalid["messages"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("code");
            }
            "attribution" => invalid["messages"][0]["spans"] = json!([]),
            "cause" => invalid["messages"][0]["message"] = "Owner has private fields".into(),
            "rendered-only" => {
                invalid["messages"][0]["rendered"] = invalid["messages"][0]["message"].clone();
                invalid["messages"][0]["message"] = "unrelated error".into();
            }
            _ => unreachable!(),
        }
        assert!(
            check_production_outcome(&source(), &invalid, &requirements).is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn semantic_contract_selection_keeps_every_snapshot_fixture_and_refuses_wrong_membership() {
    let pattern = Path::new("owner/fixtures/*.rs");
    let selected = vec![
        PathBuf::from("owner/fixtures/first.rs"),
        PathBuf::from("owner/fixtures/second.rs"),
    ];
    let contract = || ProductionSemanticDenialContract {
        fixture: "first.rs",
        errors: &[ProductionDiagnosticExpectation::Code {
            code: "E0624",
            symbols: &["Owner"],
        }],
    };
    let contracts = [contract()];
    let resolved = resolve_contracts(pattern, &selected, &contracts).unwrap();
    assert_eq!(resolved.len(), selected.len());
    assert!(resolved[0].is_some());
    assert!(
        resolved[1].is_none(),
        "the second file retains its full snapshot contract"
    );
    assert!(resolve_contracts(pattern, &selected, &[contract(), contract()]).is_err());
    for fixture in ["missing.rs", "../first.rs", "other/first.rs"] {
        assert!(resolve_contracts(
            pattern,
            &selected,
            &[ProductionSemanticDenialContract {
                fixture,
                errors: &[]
            }]
        )
        .is_err());
    }
    assert!(resolve_contracts(
        pattern,
        &[PathBuf::from("another/fixtures/first.rs")],
        &[contract()]
    )
    .is_err());
}
