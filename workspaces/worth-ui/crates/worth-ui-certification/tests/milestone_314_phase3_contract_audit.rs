use super::{milestone_314_ledger, repository_document};

const CONTRACT: &str = "plans/worth-ui/milestone-3.14-phase-3-contract.toml";
const LEDGER: &str = "plans/worth-ui/milestone-3.14-phase-3-proof-ledger.csv";
const IDS: [&str; 7] = [
    "P3-01", "P3-02", "P3-03", "P3-04", "P3-05", "P3-06", "P3-07",
];

fn inputs() -> (toml::Value, String) {
    let contract =
        toml::from_str(&repository_document(CONTRACT)).expect("Phase 3 contract should parse");
    (contract, repository_document(LEDGER))
}

fn validate(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    validate_identity(contract)?;
    validate_authority(contract)?;
    validate_destination(contract)?;
    validate_limits_and_fences(contract)?;
    validate_gates_and_ledger(contract, ledger)?;
    Ok(())
}

fn validate_identity(contract: &toml::Value) -> Result<(), String> {
    if contract["schema"].as_str() != Some("worth-ui.milestone-3.14.phase-3-contract.v1")
        || contract["milestone"].as_str() != Some("3.14")
        || contract["phase"].as_integer() != Some(3)
        || contract["ledger"].as_str() != Some(LEDGER)
        || contract["predecessor_contract"].as_str()
            != Some("plans/worth-ui/milestone-3.14-phase-2-contract.toml")
    {
        return Err("Phase 3 contract identity drifted".to_owned());
    }
    Ok(())
}

fn validate_authority(contract: &toml::Value) -> Result<(), String> {
    for (field, expected) in [
        (
            "definition_truth",
            "frozen Rust-registered UiIntentDefinition<I>",
        ),
        (
            "authored_truth",
            "canonical file or Rust-authored semantic artifact",
        ),
        (
            "route_source",
            "sealed UiIntentRouteSource::MountedInteraction",
        ),
        ("target_truth", "current mounted interaction affinity"),
        (
            "payload_truth",
            "one phase-scoped immutable input basis over declared owners",
        ),
        (
            "operability_truth",
            "runtime-derived closed axes over the same input basis",
        ),
        (
            "confirmation_truth",
            "one runtime-owned affine challenge slot",
        ),
        (
            "downstream_handoff",
            "move-only admitted intent with no provider invocation surface",
        ),
    ] {
        if contract["authority"][field].as_str() != Some(expected) {
            return Err(format!("Phase 3 authority `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_destination(contract: &toml::Value) -> Result<(), String> {
    for (field, expected) in [
        (
            "declaration_root",
            "workspaces/worth-ui/crates/worth-ui-runtime/src/declaration/intent",
        ),
        (
            "payload_root",
            "workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/intent/payload",
        ),
        (
            "operability_root",
            "workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/intent/operability",
        ),
        (
            "confirmation_root",
            "workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/intent/confirmation",
        ),
        (
            "admission_root",
            "workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/intent/admission",
        ),
        (
            "dsl_root",
            "workspaces/worth-ui/crates/worth-ui-dsl/src/semantic/intent",
        ),
        (
            "certification_root",
            "workspaces/worth-ui/crates/worth-ui-certification/tests/application_contracts/intent",
        ),
    ] {
        if contract["destination"][field].as_str() != Some(expected) {
            return Err(format!("Phase 3 destination `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_limits_and_fences(contract: &toml::Value) -> Result<(), String> {
    if contract["limits"]["maximum_payload_fields"].as_integer() != Some(64)
        || contract["limits"]["maximum_pending_challenges"].as_integer() != Some(16)
        || contract["limits"]["maximum_routes_per_application"].as_integer() != Some(65_536)
        || contract["limits"]["new_test_target"].as_bool() != Some(false)
    {
        return Err("Phase 3 limits drifted".to_owned());
    }
    let fences = contract["compile_time_enforcement"]
        .as_table()
        .ok_or_else(|| "Phase 3 compile-time fences are missing".to_owned())?;
    if fences.len() != 8 || fences.values().any(|value| value.as_bool() != Some(true)) {
        return Err("Phase 3 compile-time fences drifted".to_owned());
    }
    if contract["forbidden"]["ordinary_paths"]
        .as_array()
        .map(Vec::len)
        != Some(8)
    {
        return Err("Phase 3 forbidden-path inventory drifted".to_owned());
    }
    Ok(())
}

fn validate_gates_and_ledger(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    let status = contract["status"]
        .as_str()
        .ok_or_else(|| "Phase 3 contract status is missing".to_owned())?;
    let gates = contract["phase_gate"]
        .as_array()
        .ok_or_else(|| "Phase 3 gates are missing".to_owned())?;
    let rows = milestone_314_ledger::parse_ledger(ledger)?;
    if gates.len() != IDS.len() || rows.len() != IDS.len() {
        return Err("Phase 3 gate count drifted".to_owned());
    }
    for (index, id) in IDS.iter().enumerate() {
        if gates[index]["id"].as_str() != Some(id) || rows[index][0] != *id {
            return Err(format!("expected Phase 3 gate {id}"));
        }
        if rows[index].len() != 10
            || rows[index][7] != gates[index]["command"].as_str().unwrap_or_default()
        {
            return Err(format!("{id} evidence command drifted"));
        }
        match rows[index][8].as_str() {
            "OPEN" if rows[index][9].is_empty() => {}
            "PROVED" if rows[index][9].len() >= 80 => {}
            _ => return Err(format!("{id} status/evidence is dishonest")),
        }
    }
    let all_proved = rows.iter().all(|row| row[8] == "PROVED");
    if !matches!(status, "implementation" | "closed") || (status == "closed") != all_proved {
        return Err("Phase 3 contract status disagrees with its ledger".to_owned());
    }
    Ok(())
}

#[test]
fn milestone_314_phase3_contract_freezes_the_next_authority_progression() {
    let (contract, ledger) = inputs();
    validate(&contract, &ledger).expect("Phase 3 contract and ledger should agree");
    let predecessor: toml::Value = toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.14-phase-2-contract.toml",
    ))
    .expect("Phase 2 contract should parse");
    assert_eq!(predecessor["status"].as_str(), Some("closed"));
    assert_eq!(contract["status"].as_str(), Some("closed"));
    let rows = milestone_314_ledger::parse_ledger(&ledger).expect("Phase 3 ledger should parse");
    assert!(rows.iter().all(|row| row[8] == "PROVED"));
}

#[test]
fn milestone_314_phase3_contract_rejects_hostile_drift() {
    let (contract, ledger) = inputs();
    for (label, mutation) in hostile_contract_mutations(&contract) {
        assert!(
            validate(&mutation, &ledger).is_err(),
            "{label} mutation should fail"
        );
    }
    let mut reopened = contract;
    reopened["status"] = toml::Value::String("implementation".to_owned());
    assert!(validate(&reopened, &ledger).is_err());
}

fn hostile_contract_mutations(contract: &toml::Value) -> Vec<(&'static str, toml::Value)> {
    let mut predecessor = contract.clone();
    predecessor["predecessor_contract"] =
        toml::Value::String("plans/worth-ui/milestone-3.14-phase-1-contract.toml".to_owned());
    let mut route_source = contract.clone();
    route_source["authority"]["route_source"] =
        toml::Value::String("raw host observation".to_owned());
    let mut challenge_capacity = contract.clone();
    challenge_capacity["limits"]["maximum_pending_challenges"] = toml::Value::Integer(17);
    let mut fence = contract.clone();
    fence["compile_time_enforcement"]["challenge_is_move_only"] = toml::Value::Boolean(false);
    let mut gate_order = contract.clone();
    gate_order["phase_gate"]
        .as_array_mut()
        .expect("phase gates")[2]["id"] = toml::Value::String("P3-04".to_owned());
    vec![
        ("predecessor", predecessor),
        ("route source", route_source),
        ("challenge capacity", challenge_capacity),
        ("compile-time fence", fence),
        ("gate order", gate_order),
    ]
}
