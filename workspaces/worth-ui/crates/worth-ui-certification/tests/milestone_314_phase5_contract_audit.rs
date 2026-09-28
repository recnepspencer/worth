use super::{milestone_314_ledger, repository_document};

#[path = "milestone_314_phase5_compile_residue.rs"]
mod compile_residue;
#[path = "milestone_314_phase5_documentation.rs"]
mod documentation;

const CONTRACT: &str = "plans/worth-ui/milestone-3.14-phase-5-contract.toml";
const LEDGER: &str = "plans/worth-ui/milestone-3.14-phase-5-proof-ledger.csv";
const MAIN_LEDGER: &str = "plans/worth-ui/milestone-3.14-proof-ledger.csv";
const IDS: [&str; 8] = [
    "P5-01", "P5-02", "P5-03", "P5-04", "P5-05", "P5-06", "P5-07", "P5-08",
];

fn inputs() -> (toml::Value, String) {
    let contract =
        toml::from_str(&repository_document(CONTRACT)).expect("Phase 5 contract should parse");
    (contract, repository_document(LEDGER))
}

fn validate(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    validate_identity(contract)?;
    validate_predecessor(contract)?;
    validate_authority(contract)?;
    validate_destination(contract)?;
    validate_limits(contract)?;
    validate_fences(contract)?;
    validate_gates_and_ledger(contract, ledger)
}

fn validate_identity(contract: &toml::Value) -> Result<(), String> {
    if contract["schema"].as_str() != Some("worth-ui.milestone-3.14.phase-5-contract.v1")
        || contract["milestone"].as_str() != Some("3.14")
        || contract["phase"].as_integer() != Some(5)
        || contract["ledger"].as_str() != Some(LEDGER)
        || contract["main_ledger"].as_str() != Some(MAIN_LEDGER)
        || contract["predecessor_contract"].as_str()
            != Some("plans/worth-ui/milestone-3.14-phase-4-contract.toml")
    {
        return Err("Phase 5 contract identity drifted".to_owned());
    }
    Ok(())
}

fn validate_predecessor(contract: &toml::Value) -> Result<(), String> {
    let predecessor: toml::Value = toml::from_str(&repository_document(
        contract["predecessor_contract"]
            .as_str()
            .ok_or_else(|| "Phase 5 predecessor path is missing".to_owned())?,
    ))
    .map_err(|error| format!("Phase 4 contract should parse: {error}"))?;
    if predecessor["status"].as_str() != Some("closed")
        || predecessor["phase"].as_integer() != Some(4)
    {
        return Err("Phase 5 predecessor is not closed Phase 4".to_owned());
    }
    Ok(())
}

fn validate_authority(contract: &toml::Value) -> Result<(), String> {
    let authority = contract["authority"]
        .as_table()
        .ok_or_else(|| "Phase 5 authority table is missing".to_owned())?;
    if authority.len() != 9
        || authority
            .values()
            .any(|value| value.as_str().is_none_or(|text| text.len() < 60))
    {
        return Err("Phase 5 authority split drifted".to_owned());
    }
    Ok(())
}

fn validate_destination(contract: &toml::Value) -> Result<(), String> {
    for (field, suffix) in [
        ("pulse_root", "apps/platform-pulse/src"),
        ("pulse_intent_root", "apps/platform-pulse/src/intent"),
        ("pulse_input_root", "apps/platform-pulse/intent_samples"),
        ("pulse_observation", "observation_contract/intent.rs"),
        ("executable_world", "courtroom/platform_pulse_intent"),
        ("cost_evidence", "application_contracts/intent/cost"),
        (
            "lifecycle_evidence",
            "application_contracts/intent/lifecycle/ia_11",
        ),
        ("compile_owner", "run_worth_ui_compile_contracts.py"),
        ("documentation_root", "workspaces/worth-ui/docs"),
    ] {
        if !contract["destination"][field]
            .as_str()
            .is_some_and(|value| value.ends_with(suffix))
        {
            return Err(format!("Phase 5 destination `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_limits(contract: &toml::Value) -> Result<(), String> {
    for (field, expected) in [
        ("transition_deadline_seconds", 5),
        ("journey_ceiling_seconds", 45),
        ("ordinary_warm_ceiling_seconds", 60),
        ("compile_sessions", 2),
    ] {
        if contract["limits"][field].as_integer() != Some(expected) {
            return Err(format!("Phase 5 limit `{field}` drifted"));
        }
    }
    for (field, expected) in [
        ("route_counts", &[1, 1024, 65_536][..]),
        ("interaction_counts", &[0, 1, 16][..]),
        ("payload_widths", &[0, 1, 64][..]),
        ("queue_occupancies", &[0, 15, 16, 17][..]),
    ] {
        let observed = contract["limits"][field]
            .as_array()
            .ok_or_else(|| format!("Phase 5 limit `{field}` is missing"))?
            .iter()
            .map(|value| value.as_integer().unwrap_or_default())
            .collect::<Vec<_>>();
        if observed != expected {
            return Err(format!("Phase 5 limit axis `{field}` drifted"));
        }
    }
    for field in ["new_test_target", "new_binary", "nested_cargo_invocation"] {
        if contract["limits"][field].as_bool() != Some(false) {
            return Err(format!("Phase 5 topology limit `{field}` drifted"));
        }
    }
    Ok(())
}

fn validate_fences(contract: &toml::Value) -> Result<(), String> {
    let fences = contract["compile_time_enforcement"]
        .as_table()
        .ok_or_else(|| "Phase 5 compile-time fences are missing".to_owned())?;
    if fences.len() != 10 || fences.values().any(|value| value.as_bool() != Some(true)) {
        return Err("Phase 5 compile-time fences drifted".to_owned());
    }
    if contract["forbidden"]["ordinary_paths"]
        .as_array()
        .map(Vec::len)
        != Some(12)
    {
        return Err("Phase 5 forbidden-path inventory drifted".to_owned());
    }
    Ok(())
}

fn validate_gates_and_ledger(contract: &toml::Value, ledger: &str) -> Result<(), String> {
    let status = contract["status"]
        .as_str()
        .ok_or_else(|| "Phase 5 contract status is missing".to_owned())?;
    let gates = contract["phase_gate"]
        .as_array()
        .ok_or_else(|| "Phase 5 gates are missing".to_owned())?;
    let rows = milestone_314_ledger::parse_ledger(ledger)?;
    if gates.len() != IDS.len() || rows.len() != IDS.len() {
        return Err("Phase 5 gate count drifted".to_owned());
    }
    let mut reached_open = false;
    for (index, id) in IDS.iter().enumerate() {
        if gates[index]["id"].as_str() != Some(id)
            || rows[index][0] != *id
            || rows[index].len() != 10
            || rows[index][7] != gates[index]["command"].as_str().unwrap_or_default()
        {
            return Err(format!("Phase 5 gate `{id}` drifted"));
        }
        match rows[index][8].as_str() {
            "OPEN" if rows[index][9].is_empty() => reached_open = true,
            "PROVED" if rows[index][9].len() >= 80 && !reached_open => {}
            "PROVED" if reached_open => {
                return Err(format!("{id} is proved after an open predecessor gate"));
            }
            _ => return Err(format!("{id} status or evidence is dishonest")),
        }
    }
    let all_proved = rows.iter().all(|row| row[8] == "PROVED");
    if !matches!(status, "implementation" | "closed") || (status == "closed") != all_proved {
        return Err("Phase 5 contract status disagrees with its ledger".to_owned());
    }
    Ok(())
}

#[test]
fn milestone_314_phase5_contract_freezes_product_world_closure() {
    let (contract, ledger) = inputs();
    validate(&contract, &ledger).expect("Phase 5 contract and ledger should agree");
    let phase_1: toml::Value = toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.14-phase-1-contract.toml",
    ))
    .expect("Phase 1 contract should parse");
    milestone_314_ledger::validate_at_phase(
        &phase_1,
        &repository_document(MAIN_LEDGER),
        milestone_314_ledger::CURRENT_IMPLEMENTATION_PHASE,
    )
    .expect("main IA ledger should admit Phase 5 ownership without premature closure");
}

#[test]
fn milestone_314_phase5_contract_rejects_hostile_drift() {
    let (contract, ledger) = inputs();
    for (label, mutation) in hostile_contract_mutations(&contract) {
        assert!(validate(&mutation, &ledger).is_err(), "{label} should fail");
    }
    let mut skipped = milestone_314_ledger::parse_ledger(&ledger).expect("Phase 5 ledger parses");
    skipped[0][8] = "OPEN".to_owned();
    skipped[0][9].clear();
    skipped[1][8] = "PROVED".to_owned();
    skipped[1][9] = "hostile evidence ".repeat(8);
    assert!(validate(&contract, &milestone_314_ledger::render_ledger(&skipped)).is_err());
}

fn hostile_contract_mutations(contract: &toml::Value) -> Vec<(&'static str, toml::Value)> {
    let mut predecessor = contract.clone();
    predecessor["predecessor_contract"] =
        toml::Value::String("plans/worth-ui/milestone-3.14-phase-3-contract.toml".to_owned());
    let mut authority = contract.clone();
    authority["authority"]["query_truth"] = toml::Value::String("UI mutates Query".to_owned());
    let mut limit = contract.clone();
    limit["limits"]["journey_ceiling_seconds"] = toml::Value::Integer(46);
    let mut topology = contract.clone();
    topology["limits"]["new_test_target"] = toml::Value::Boolean(true);
    let mut fence = contract.clone();
    fence["compile_time_enforcement"]["diagnostic_trace_has_no_operational_conversion"] =
        toml::Value::Boolean(false);
    let mut forbidden = contract.clone();
    forbidden["forbidden"]["ordinary_paths"]
        .as_array_mut()
        .expect("forbidden paths")
        .pop();
    let mut gate = contract.clone();
    gate["phase_gate"].as_array_mut().expect("phase gates")[3]["id"] =
        toml::Value::String("P5-05".to_owned());
    vec![
        ("predecessor mutation", predecessor),
        ("authority mutation", authority),
        ("journey limit mutation", limit),
        ("topology mutation", topology),
        ("compile fence mutation", fence),
        ("forbidden-path mutation", forbidden),
        ("gate-order mutation", gate),
    ]
}
