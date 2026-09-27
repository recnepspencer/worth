use std::collections::BTreeSet;

use crate::repository_document;

#[test]
fn phase_2_scenario_contracts_have_every_required_evidence_field() {
    let contract = phase_2_contract();
    assert_eq!(contract["status"].as_str(), Some("closed"));
    let scenarios = contract["scenario"]
        .as_array()
        .expect("Phase 2 scenarios are an array");
    let ids = scenarios
        .iter()
        .map(|scenario| scenario["id"].as_str().expect("scenario id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from(["VS-02", "VS-05", "VS-06", "VS-07"]));
    for scenario in scenarios {
        for field in REQUIRED_SCENARIO_TEXT_FIELDS {
            assert!(
                scenario[field]
                    .as_str()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{} has no {field}",
                scenario["id"]
            );
        }
        assert!(
            scenario["mutation_control"]
                .as_array()
                .is_some_and(|controls| controls.len() >= 4),
            "{} needs at least four mutation controls",
            scenario["id"]
        );
    }
}

#[test]
fn phase_2_ledger_rows_are_unique_and_status_evidence_agree() {
    let ledger = repository_document("plans/worth-ui/milestone-3.11-phase-2-proof-ledger.csv");
    let mut ids = BTreeSet::new();
    let mut rows = 0;
    for line in ledger.lines().skip(1) {
        let id = line.split(',').next().expect("ledger row id");
        assert!(ids.insert(id.to_owned()), "duplicate ledger row {id}");
        assert!(
            line.contains(",\"PROVED\","),
            "{id} is not closed as PROVED"
        );
        assert!(!line.ends_with(",\"\""), "{id} is proved without evidence");
        rows += 1;
    }
    assert_eq!(rows, 20);
    assert_eq!(ids.first().map(String::as_str), Some("P2-01"));
    assert_eq!(ids.last().map(String::as_str), Some("P2-20"));

    let final_row = ledger
        .lines()
        .find(|line| line.starts_with("P2-20,"))
        .expect("P2-20 closure row");
    for marker in REQUIRED_FINAL_EVIDENCE_MARKERS {
        assert!(
            final_row.contains(marker),
            "P2-20 is missing final evidence marker {marker}"
        );
    }
}

fn phase_2_contract() -> toml::Value {
    let text = repository_document("plans/worth-ui/milestone-3.11-phase-2-contract.toml");
    toml::from_str(&text).expect("Phase 2 contract is TOML")
}

const REQUIRED_SCENARIO_TEXT_FIELDS: &[&str] = &[
    "production_claim",
    "plausible_defect",
    "world",
    "semantic_handles",
    "production_entry",
    "authority_boundary",
    "action_schedule",
    "typed_expected_outcome",
    "consequential_state",
    "independent_oracle",
    "cleanup_disposition",
    "structural_cost",
    "owning_cost_lane",
    "command",
];

const REQUIRED_FINAL_EVIDENCE_MARKERS: &[&str] = &[
    "858 warnings-denied runtime tests",
    "27 focused visual snapshot tests",
    "186 application contracts",
    "120 topology contracts",
    "30 fail targets",
    "13 pass targets",
    "2 Cargo sessions",
    "32 test-topology meta-tests",
    "workspace clippy -D warnings",
    "3342-file Rust line-cap audit",
    "boundary-check",
    "agent-context check",
    "closed 20-row Phase 2 ledger audit",
];
