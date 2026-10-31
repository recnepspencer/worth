//! Actual source readmission is distinct from live settlement and adoption.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Artifact;
use crate::facade::durability::{
    RelationalAssociatedEffectRefusal as Refusal,
    RelationalAssociatedEffectSettlement as Settlement, RelationalCheckpointArtifactPolicy,
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
fn associated_effects_read_exact_source_and_report_actual_pending_and_settled_routes() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let association = Association::from_bytes(b"actual-original-owner-attempt").unwrap();
    let performed = perform(execution, &runtime, "source-backed", &association);
    let original_receipt = performed.canonical_commit().commit.clone();
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    let owner = runtime.durability_authority();
    let mut subjects = owner
        .discover_associated_effects(execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    let subject = subjects.pop().unwrap();
    let pending = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(pending.settlement(), Settlement::PerformedPending);
    assert_eq!(pending.subject().receipt(), &original_receipt);
    assert_eq!(pending.subject().position(), performed.patch_position());
    assert_eq!(
        pending.subject().artifact_digest(),
        artifact.payload_digest().as_bytes()
    );
    assert_eq!(
        pending.semantic_result(),
        performed.canonical_commit().semantic_result()
    );
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let settled = owner
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(settled.settlement(), Settlement::PerformedSettled);
    assert_eq!(settled.semantic_result(), committed.semantic_result());
    let foreign = durable_runtime_with_test_schema();
    assert!(matches!(
        foreign
            .durability_authority()
            .readmit_associated_effect(execution, &subject),
        Err(Refusal::ForeignRuntime)
    ));
    // An already-resident body must not hide damage in the actual selected source.
    let medium = attached_test_medium(&runtime);
    medium.flip_artifact_byte(artifact.position(), artifact.as_bytes().len() - 1);
    assert!(matches!(
        owner.readmit_associated_effect(execution, &subject),
        Err(Refusal::Recovery(_))
    ));
    assert_eq!(
        subject.receipt(),
        &original_receipt,
        "failed readmission leaves original evidence owned"
    );
    release_test_commit_snapshot(&runtime, &committed);
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
    let runtime = durable_runtime_with_test_schema();
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
    let runtime = durable_runtime_with_test_schema();
    let association = Association::from_bytes(b"one-selected-cold-original").unwrap();
    let committed = runtime
        .settle_performed_publication(perform(execution, &runtime, "selected", &association))
        .unwrap();
    let selected_id = committed.commit.commit_id;
    let semantic = committed.semantic_result().clone();
    release_test_commit_snapshot(&runtime, &committed);
    create_entity(execution, &runtime, "unrelated-cold");
    let unrelated = attached_test_medium(&runtime).offered().pop().unwrap();
    create_entity(execution, &runtime, "checkpoint-head");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    attached_test_medium(&runtime).file_checkpoint(checkpoint);
    let (_, recovered) = recover_with(execution, &runtime, durable_runtime_with_test_schema);
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
    attached_test_medium(&recovered).drop_artifact(unrelated.position());
    let owner = recovered.durability_authority();
    let subjects = owner
        .discover_associated_effects(execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    assert!(
        selected.positioned_if_resident().is_none(),
        "discovery reads only metadata"
    );
    let effect = owner
        .readmit_associated_effect(execution, &subjects[0])
        .unwrap();
    assert_eq!(effect.settlement(), Settlement::CheckpointReadmitted);
    assert_eq!(effect.semantic_result(), &semantic);
    assert!(unrelated_cell.positioned_if_resident().is_none());
    let selected_frame = Artifact::readmit(
        &attached_test_medium(&recovered)
            .artifact_bytes(effect.subject().position())
            .unwrap(),
        execution,
    )
    .unwrap();
    assert_eq!(selected_frame.envelope().semantic_result(), &semantic);
}

#[test]
fn associated_effects_observers_do_not_keep_the_closed_source_owner_alive() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let association = Association::from_bytes(b"descriptive-observer-after-owner-close").unwrap();
    let committed = runtime
        .settle_performed_publication(perform(execution, &runtime, "observer", &association))
        .unwrap();
    let receipt = committed.commit.clone();
    release_test_commit_snapshot(&runtime, &committed);
    drop(committed);
    let source = attached_durable_medium(&runtime).unwrap();
    let weak = std::sync::Arc::downgrade(&source);
    drop(source);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    runtime.prepare_close().unwrap().finish();
    assert!(
        weak.upgrade().is_none(),
        "description observers cannot retain the actual source/Serving owner"
    );
    assert_eq!(subject.receipt(), &receipt);
    assert_eq!(effect.semantic_result().receipt(), &receipt);
}
