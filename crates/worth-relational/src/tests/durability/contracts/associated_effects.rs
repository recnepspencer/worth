//! Actual source readmission is distinct from live settlement and adoption.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::facade::durability::{
    RelationalAssociatedEffectRefusal as Refusal,
    RelationalAssociatedEffectSettlement as Settlement,
};
use crate::facade::mvcc::{
    RelationalCallerAssociation as Association, RelationalPublicationOutcome,
};
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

pub(super) fn perform(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    association: &Association,
) -> crate::mvcc::PerformedRelationalCommit {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("actual candidate performs")
    };
    performed
}

#[test]
fn associated_effects_refuse_memory_and_absent_settlement_evidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let association = Association::from_bytes(b"memory-is-not-a-durable-effect").unwrap();
    let memory = runtime_with_test_schema();
    let committed = memory
        .settle_performed_publication(perform(execution, &memory, "memory", &association))
        .unwrap();
    assert!(matches!(
        memory
            .durability_authority()
            .discover_associated_effects(execution, &association),
        Err(Refusal::AssociatedMemory { .. })
    ));
    release_test_commit_snapshot(&memory, &committed);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let committed = runtime
        .settle_performed_publication(perform(execution, &runtime, "lost-route", &association))
        .unwrap();
    let owner = runtime.durability_authority();
    let subject = owner
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    runtime
        .history
        .remove_canonical_publication_route_for_test(committed.commit.commit_id);
    assert!(matches!(
        owner.readmit_associated_effect(execution, &subject),
        Err(Refusal::MissingSettlementEvidence)
    ));
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn associated_effects_discover_cold_subject_without_reading_unrelated_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let association = Association::from_bytes(b"one-selected-cold-original").unwrap();
    let committed = runtime
        .settle_performed_publication(perform(execution, &runtime, "selected", &association))
        .unwrap();
    let selected_id = committed.commit.commit_id;
    let selected_pos = committed.patch_position();
    let semantic = committed.semantic_result();
    release_test_commit_snapshot(&runtime, &committed);
    create_entity(execution, &runtime, "unrelated-cold");
    let unrelated = medium.latest_committed_artifact().unwrap();
    create_entity(execution, &runtime, "checkpoint-head");
    capture_and_file_checkpoint(execution, &runtime, &medium);
    let mut recovered = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(execution, &mut recovered, &medium);
    let selected = recovered
        .history
        .commit_artifact(selected_id)
        .unwrap()
        .envelope_residency();
    let unrelated_cell = recovered
        .history
        .commit_artifact(unrelated.commit_id())
        .unwrap()
        .envelope_residency();
    assert!(selected.positioned_if_resident().is_none());
    assert!(unrelated_cell.positioned_if_resident().is_none());
    let unrelated_sha = medium.drop_artifact_and_member(unrelated.position());
    let selected_sha = medium.artifact_sha256(selected_pos);
    medium.start_recording_member_reads();
    let owner = recovered.durability_authority();
    let subjects = owner
        .discover_associated_effects(execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    assert!(
        medium.member_read_digests().is_empty(),
        "discovery must perform zero protected member reads"
    );
    assert!(
        selected.positioned_if_resident().is_none(),
        "discovery reads only metadata"
    );
    let effect = owner
        .readmit_associated_effect(execution, &subjects[0])
        .unwrap();
    assert_eq!(effect.settlement(), Settlement::CheckpointReadmitted);
    assert_eq!(effect.semantic_result(), semantic);
    assert_eq!(
        medium.member_read_digests(),
        vec![selected_sha],
        "readmission must read only the selected commit member"
    );
    assert!(!medium.member_read_digests().contains(&unrelated_sha));
    assert!(unrelated_cell.positioned_if_resident().is_none());
    let selected_frame = Artifact::readmit(
        &medium.artifact_bytes(effect.subject().position()).unwrap(),
        execution,
    )
    .unwrap();
    assert_eq!(selected_frame.envelope().semantic_result(), semantic);
}
