use crate::milestone_312_ledger as ledger;
use crate::{repository_document, workspace_source_inventory};

fn phase_4_contract() -> toml::Value {
    toml::from_str(&repository_document(
        "plans/worth-ui/milestone-3.12-phase-4-contract.toml",
    ))
    .expect("Milestone 3.12 Phase 4 contract is TOML")
}

#[test]
fn milestone_312_phase4_contract_and_ledger_are_exact() {
    let contract = phase_4_contract();
    assert_eq!(
        contract["schema"].as_str(),
        Some("worth-ui.milestone-3.12.phase-4-contract.v1")
    );
    assert_eq!(contract["milestone"].as_str(), Some("3.12"));
    assert_eq!(contract["phase"].as_integer(), Some(4));
    assert!(matches!(
        contract["status"].as_str(),
        Some("implementation" | "closed")
    ));
    let ledger_text = repository_document("plans/worth-ui/milestone-3.12-phase-4-proof-ledger.csv");
    ledger::validate_phase_4(&contract, &ledger_text)
        .unwrap_or_else(|failure| panic!("Phase 4 ledger is invalid: {failure}"));
}

#[test]
fn milestone_312_phase4_native_source_rebind_is_the_only_ordinary_mutation_root() {
    let inventory = workspace_source_inventory();
    assert!(!inventory
        .contains("crates/worth-ui-runtime/src/facade/entry/native_application_replacement.rs"));
    let shell = inventory.text("crates/worth-ui-runtime/src/facade/entry/native_source_rebind.rs");
    assert!(shell.contains("pub fn begin_source_rebind("));
    assert!(shell.contains(".begin_observation_turn()"));
    assert!(shell.contains(".prepare_rebind("));
    let pulse = inventory.text("apps/platform-pulse/src/native_frame/rebind.rs");
    assert!(pulse.contains(".begin_source_rebind("));
    assert!(!pulse.contains(".replace_application("));
    assert!(inventory
        .rust_files_under("crates/worth-ui-runtime/src/facade")
        .all(|source| !source.text().contains("pub fn replace_application(")));
}

#[test]
fn milestone_312_phase4_protocol_guarantees_survive_current_v9_evolution() {
    let envelope = workspace_source_inventory()
        .text("apps/platform-pulse/src/observation_contract/envelope.rs");
    assert!(envelope.contains("PLATFORM_PULSE_LIFECYCLE_OBSERVATION_SCHEMA_VERSION: u16 = 9"));
    assert!(envelope.contains("CompleteV9"));
    assert!(envelope.contains("InheritedLifecycleOnly"));
    assert!(envelope.contains("schema_version @ 2..=8"));
    assert!(!envelope.contains("failure-artifact.v1"));
    let failure = workspace_source_inventory()
        .text("apps/platform-pulse/tests/executable_world/failure_teardown/retained_artifact.rs");
    assert!(failure.contains("worth-ui.platform-pulse.failure-artifact.v1"));
}
