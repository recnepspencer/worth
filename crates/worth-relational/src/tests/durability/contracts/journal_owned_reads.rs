//! Uncheckpointed journal envelopes charge their actual record bytes on demand
//! and export, never the bounded maximum artifact reservation.
use crate::publication::patch::data::patch_position::PatchStreamPosition;
use crate::tests::support::durability::OwnedTestDurableMedium;
use crate::tests::support::*;

#[test]
fn journal_envelope_demand_and_export_charge_actual_bytes_under_sub_256mib_budget() {
    let setup_host = crate::test_execution_policy::serial_request();
    let setup = worth_execution::ExecutionRequest::serial(&setup_host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let entity = create_entity(&setup, &runtime, "tight-budget-0");
    update_entity_and_release_snapshot(&setup, &runtime, entity, "tight-budget-1");
    let mut recovered = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(&setup, &mut recovered, &medium);
    let receipt = recovered
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(1))
        .unwrap();
    let budget = worth_execution::SerialMemoryBudget::new(Some(16 * 1024 * 1024));
    let tight_host = crate::test_execution_policy::serial_request_with_memory(budget);
    let tight = worth_execution::ExecutionRequest::serial(&tight_host);
    let demanded = recovered
        .history()
        .demand_commit_envelope(&tight, receipt.commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(demanded.commit, receipt);
    let exported = recovered
        .history
        .commit_artifact(receipt.commit_id)
        .unwrap()
        .envelope_residency();
    let exported =
        crate::durability::canonical_commit_artifact::retained_frame_reads::export_artifact(
            &exported, &tight,
        )
        .unwrap();
    assert_eq!(exported.position(), PatchStreamPosition(1));
}
