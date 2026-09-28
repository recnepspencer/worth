use std::collections::BTreeSet;

use crate::repository_document;

#[test]
fn phase_3_scenario_contracts_have_every_required_evidence_field() {
    let contract = phase_3_contract();
    assert_eq!(contract["status"].as_str(), Some("implementation"));
    let scenarios = contract["scenario"]
        .as_array()
        .expect("Phase 3 scenarios are an array");
    let ids = scenarios
        .iter()
        .map(|scenario| scenario["id"].as_str().expect("scenario id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from(["VS-03", "VS-04", "VS-05", "VS-08"]));
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
fn phase_3_ledger_rows_are_unique_and_status_evidence_agree() {
    let ledger = repository_document("plans/worth-ui/milestone-3.11-phase-3-proof-ledger.csv");
    let mut ids = BTreeSet::new();
    let mut rows = 0;
    for line in ledger.lines().skip(1) {
        let id = line.split(',').next().expect("ledger row id");
        assert!(ids.insert(id.to_owned()), "duplicate ledger row {id}");
        let proved = line.contains(",\"PROVED\",");
        let open = line.contains(",\"OPEN\",");
        assert!(proved ^ open, "{id} has an invalid status");
        if proved {
            assert!(!line.ends_with(",\"\""), "{id} is proved without evidence");
        } else {
            assert!(line.ends_with(",\"\""), "{id} is open but claims evidence");
        }
        rows += 1;
    }
    assert_eq!(rows, 20);
    assert_eq!(ids.first().map(String::as_str), Some("P3-01"));
    assert_eq!(ids.last().map(String::as_str), Some("P3-20"));
}

fn phase_3_contract() -> toml::Value {
    let text = repository_document("plans/worth-ui/milestone-3.11-phase-3-contract.toml");
    toml::from_str(&text).expect("Phase 3 contract is TOML")
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
