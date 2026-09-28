use super::{milestone_313_ledger, repository_document, workspace_source_inventory};

fn phase_1_inputs() -> (toml::Value, String) {
    let contract_text = repository_document("plans/worth-ui/milestone-3.13-phase-1-contract.toml");
    let contract = toml::from_str(&contract_text).expect("Phase 1 contract should parse");
    let ledger = repository_document("plans/worth-ui/milestone-3.13-proof-ledger.csv");
    (contract, ledger)
}

fn assert_rejected(contract: &toml::Value, ledger: &str, phase: i64, label: &str) {
    assert!(
        milestone_313_ledger::validate_at_phase(contract, ledger, phase).is_err(),
        "{label} mutation should be rejected"
    );
}

#[test]
fn milestone_313_contract_and_current_phase_5_ledger_are_exact() {
    let (contract, ledger) = phase_1_inputs();

    milestone_313_ledger::validate_at_phase(&contract, &ledger, 5)
        .expect("the frozen contract and current Phase 5 ledger should agree");
}

#[test]
fn milestone_313_historical_phase_1_posture_rejects_any_closed_row() {
    let (contract, ledger) = phase_1_inputs();
    let mut rows = milestone_313_ledger::parse_ledger(&ledger).expect("ledger should parse");
    for row in &mut rows {
        row[8] = "OPEN".to_owned();
        row[9].clear();
    }
    let open = milestone_313_ledger::render_ledger(&rows);
    milestone_313_ledger::validate_phase_1(&contract, &open)
        .expect("the historical Phase 1 posture keeps every later proof open");

    rows[1][8] = "PROVED".to_owned();
    rows[1][9] =
        "A deliberately long but premature Phase 2 proof cannot close in Phase 1.".to_owned();
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&rows),
        1,
        "historical premature proof",
    );
}

#[test]
fn milestone_313_phase1_ledger_rejects_hostile_structure_mutations() {
    let (contract, ledger) = phase_1_inputs();
    let rows = milestone_313_ledger::parse_ledger(&ledger).expect("ledger should parse");

    let mut missing = rows.clone();
    missing.remove(4);
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&missing),
        5,
        "missing row",
    );

    let mut duplicate = rows.clone();
    duplicate[4] = duplicate[3].clone();
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&duplicate),
        5,
        "duplicate row",
    );

    let mut reordered = rows.clone();
    reordered.swap(4, 5);
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&reordered),
        5,
        "reordered row",
    );

    let mut fabricated = rows;
    fabricated[4][0] = "QP-99".to_owned();
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&fabricated),
        5,
        "fabricated row",
    );
}

#[test]
fn milestone_313_phase1_ledger_rejects_premature_or_fabricated_closure() {
    let (contract, ledger) = phase_1_inputs();
    let rows = milestone_313_ledger::parse_ledger(&ledger).expect("ledger should parse");

    let mut command_drift = rows.clone();
    command_drift[0][7].push_str(" --ignored");
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&command_drift),
        5,
        "command drift",
    );

    let mut premature = rows.clone();
    premature[8][8] = "PROVED".to_owned();
    premature[8][9] =
        "A deliberately long but premature Phase 5 proof claim cannot close during Phase 4."
            .to_owned();
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&premature),
        4,
        "premature proof",
    );

    let mut fabricated = rows;
    fabricated[8][9] = "source=fiction; result=passed".to_owned();
    assert_rejected(
        &contract,
        &milestone_313_ledger::render_ledger(&fabricated),
        5,
        "evidence on an open row",
    );
}

#[test]
fn milestone_313_phase1_topology_and_dependency_owners_exist() {
    let (contract, _) = phase_1_inputs();
    let inventory = workspace_source_inventory();
    let paths = contract["required_path"]
        .as_array()
        .expect("required paths should be an array");

    for path in paths {
        let relative = path["path"].as_str().expect("required path");
        assert!(
            inventory
                .root()
                .join(relative.strip_prefix("workspaces/worth-ui/").unwrap())
                .exists(),
            "required Phase 1 owner path should exist: {relative}"
        );
    }
    assert_eq!(
        contract["ownership"]["raw_query_owner"].as_str(),
        Some("worth-ui-query-binding")
    );
    assert_eq!(
        contract["public_contract"]["raw_query_returned"].as_bool(),
        Some(false)
    );
}

#[test]
fn milestone_313_phase1_pulse_allowlist_names_only_audience_safe_query_edges() {
    let config_text = repository_document("tools/boundary-check/config/road1.toml");
    let config: toml::Value = toml::from_str(&config_text).expect("boundary config should parse");
    let allowlist = config["source_dependency_allowlists"]
        .as_array()
        .expect("source dependency allowlists")
        .iter()
        .find(|rule| {
            rule["sources"].as_array().is_some_and(|sources| {
                sources
                    .iter()
                    .any(|source| source.as_str() == Some("worth-ui-platform-pulse"))
            })
        })
        .expect("Pulse allowlist");
    let allowed = allowlist["allowed_targets"]
        .as_array()
        .expect("Pulse allowed targets")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();

    for required in ["worth-query-decl", "worth-query-host"] {
        assert!(
            allowed.contains(&required),
            "missing Pulse audience edge {required}"
        );
    }
    for forbidden in [
        "worth-query",
        "worth-query-replay",
        "worth-ui-query-binding",
        "worth-ui-runtime",
        "worth-ui-certification",
        "worth-ui-test-support",
    ] {
        assert!(
            !allowed.contains(&forbidden),
            "Pulse allowlist admits forbidden edge {forbidden}"
        );
    }
}
