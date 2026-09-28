use super::{milestone_314_ledger, repository_document};

const CONTRACT: &str = "plans/worth-ui/milestone-3.14-phase-4-contract.toml";
const LEDGER: &str = "plans/worth-ui/milestone-3.14-phase-4-proof-ledger.csv";
const IDS: [&str; 7] = [
    "P4-01", "P4-02", "P4-03", "P4-04", "P4-05", "P4-06", "P4-07",
];

fn inputs() -> (toml::Value, String) {
    let contract =
        toml::from_str(&repository_document(CONTRACT)).expect("Phase 4 contract should parse");
    (contract, repository_document(LEDGER))
}

fn validate(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    validate_identity(contract)?;
    validate_authority(contract)?;
    validate_destination(contract)?;
    validate_limits_and_fences(contract)?;
    validate_gates_and_ledger(contract, ledger)
}

fn validate_identity(contract: &toml::Value) -> Result<(), String> {
    if contract["schema"].as_str() != Some("worth-ui.milestone-3.14.phase-4-contract.v1")
        || contract["milestone"].as_str() != Some("3.14")
        || contract["phase"].as_integer() != Some(4)
        || contract["ledger"].as_str() != Some(LEDGER)
        || contract["predecessor_contract"].as_str()
            != Some("plans/worth-ui/milestone-3.14-phase-3-contract.toml")
    {
        return Err("Phase 4 contract identity drifted".to_owned());
    }
    Ok(())
}

fn validate_authority(contract: &toml::Value) -> Result<(), String> {
    for field in [
        "admission_truth",
        "provider_truth",
        "reservation_truth",
        "effect_truth",
        "settlement_truth",
        "recovery_truth",
        "consequence_truth",
        "downstream_handoff",
    ] {
        if contract["authority"][field]
            .as_str()
            .is_none_or(|value| value.len() < 40)
        {
            return Err(format!("Phase 4 authority `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_destination(contract: &toml::Value) -> Result<(), String> {
    for (field, suffix) in [
        ("execution_root", "runtime/intent_execution"),
        ("provider_root", "runtime/intent_execution/provider"),
        ("fact_root", "fact_contract/produced/intent.rs"),
        ("posture_root", "mounting/projection/intent_posture"),
        ("declaration_root", "declaration/intent"),
        ("facade_root", "worth-ui/src/facade/intent.rs"),
        (
            "certification_root",
            "application_contracts/intent/execution",
        ),
    ] {
        if !contract["destination"][field]
            .as_str()
            .is_some_and(|value| value.ends_with(suffix))
        {
            return Err(format!("Phase 4 destination `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_limits_and_fences(contract: &toml::Value) -> Result<(), String> {
    for (field, expected) in [
        ("maximum_application_attempts", 16),
        ("maximum_provider_attempts", 16),
        ("maximum_intent_attempts", 16),
        ("maximum_retained_payload_bytes", 4_194_304),
        ("maximum_consequences_per_attempt", 16),
        ("maximum_recovery_handles", 16),
        ("ordering_permutations", 24),
    ] {
        if contract["limits"][field].as_integer() != Some(expected) {
            return Err(format!("Phase 4 limit `{field}` drifted"));
        }
    }
    if contract["limits"]["new_test_target"].as_bool() != Some(false) {
        return Err("Phase 4 evidence topology drifted".to_owned());
    }
    let fences = contract["compile_time_enforcement"]
        .as_table()
        .ok_or_else(|| "Phase 4 compile-time fences are missing".to_owned())?;
    if fences.len() != 10 || fences.values().any(|value| value.as_bool() != Some(true)) {
        return Err("Phase 4 compile-time fences drifted".to_owned());
    }
    if contract["forbidden"]["ordinary_paths"]
        .as_array()
        .map(Vec::len)
        != Some(10)
    {
        return Err("Phase 4 forbidden-path inventory drifted".to_owned());
    }
    Ok(())
}

fn validate_gates_and_ledger(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    let status = contract["status"]
        .as_str()
        .ok_or_else(|| "Phase 4 contract status is missing".to_owned())?;
    let gates = contract["phase_gate"]
        .as_array()
        .ok_or_else(|| "Phase 4 gates are missing".to_owned())?;
    let rows = milestone_314_ledger::parse_ledger(ledger)?;
    if gates.len() != IDS.len() || rows.len() != IDS.len() {
        return Err("Phase 4 gate count drifted".to_owned());
    }
    let mut reached_open_gate = false;
    for (index, id) in IDS.iter().enumerate() {
        if gates[index]["id"].as_str() != Some(id) || rows[index][0] != *id {
            return Err(format!("expected Phase 4 gate {id}"));
        }
        if rows[index].len() != 10
            || rows[index][7] != gates[index]["command"].as_str().unwrap_or_default()
        {
            return Err(format!("{id} evidence command drifted"));
        }
        match rows[index][8].as_str() {
            "OPEN" if rows[index][9].is_empty() || rows[index][9].len() >= 80 => {
                reached_open_gate = true
            }
            "PROVED" if rows[index][9].len() >= 80 && !reached_open_gate => {}
            "PROVED" if reached_open_gate => {
                return Err(format!("{id} is proved after an open predecessor gate"));
            }
            _ => return Err(format!("{id} status/evidence is dishonest")),
        }
    }
    let all_proved = rows.iter().all(|row| row[8] == "PROVED");
    if !matches!(status, "implementation" | "closed") || (status == "closed") != all_proved {
        return Err("Phase 4 contract status disagrees with its ledger".to_owned());
    }
    Ok(())
}

#[test]
fn milestone_314_phase4_contract_freezes_managed_execution_authority() {
    let (contract, ledger) = inputs();
    validate(&contract, &ledger).expect("Phase 4 contract and ledger should agree");
    let predecessor: toml::Value = toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.14-phase-3-contract.toml",
    ))
    .expect("Phase 3 contract should parse");
    assert_eq!(predecessor["status"].as_str(), Some("closed"));
    let phase_1: toml::Value = toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.14-phase-1-contract.toml",
    ))
    .expect("Phase 1 contract should parse");
    milestone_314_ledger::validate_at_phase(
        &phase_1,
        &repository_document("plans/worth-ui/milestone-3.14-proof-ledger.csv"),
        milestone_314_ledger::CURRENT_IMPLEMENTATION_PHASE,
    )
    .expect("main IA ledger should admit Phase 4 ownership without claiming closure");
}

#[test]
fn milestone_314_phase4_contract_rejects_hostile_drift() {
    let (contract, ledger) = inputs();
    for (label, mutation) in hostile_contract_mutations(&contract) {
        assert!(
            validate(&mutation, &ledger).is_err(),
            "{label} mutation should fail"
        );
    }
    let mut skipped = milestone_314_ledger::parse_ledger(&ledger).expect("Phase 4 ledger parses");
    skipped[0][8] = "OPEN".to_owned();
    skipped[0][9].clear();
    skipped[1][8] = "PROVED".to_owned();
    skipped[1][9] = "hostile evidence ".repeat(8);
    assert!(validate(&contract, &milestone_314_ledger::render_ledger(&skipped)).is_err());
    let mut weak_open = milestone_314_ledger::parse_ledger(&ledger).expect("Phase 4 ledger parses");
    weak_open[6][8] = "OPEN".to_owned();
    weak_open[6][9] = "trust me".to_owned();
    assert!(validate(&contract, &milestone_314_ledger::render_ledger(&weak_open)).is_err());
}

fn hostile_contract_mutations(contract: &toml::Value) -> Vec<(&'static str, toml::Value)> {
    let mut predecessor = contract.clone();
    predecessor["predecessor_contract"] =
        toml::Value::String("plans/worth-ui/milestone-3.14-phase-2-contract.toml".to_owned());
    let mut provider = contract.clone();
    provider["authority"]["provider_truth"] = toml::Value::String("string callback".to_owned());
    let mut capacity = contract.clone();
    capacity["limits"]["maximum_application_attempts"] = toml::Value::Integer(17);
    let mut fence = contract.clone();
    fence["compile_time_enforcement"]["provider_and_definition_share_intent_type"] =
        toml::Value::Boolean(false);
    let mut forbidden = contract.clone();
    forbidden["forbidden"]["ordinary_paths"]
        .as_array_mut()
        .expect("forbidden paths")
        .pop();
    let mut gate_order = contract.clone();
    gate_order["phase_gate"]
        .as_array_mut()
        .expect("phase gates")[2]["id"] = toml::Value::String("P4-04".to_owned());
    vec![
        ("predecessor", predecessor),
        ("provider authority", provider),
        ("application capacity", capacity),
        ("compile-time fence", fence),
        ("forbidden path", forbidden),
        ("gate order", gate_order),
    ]
}
