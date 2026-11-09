//! Current/cold reads and an original retained observation share genuine record meaning.
use super::{
    current_publications::create_entity, current_read_basis::main_selected_version,
    full_read_inventory::FullReadDoor as ReadDoor, immutable_recovery_input::copy_input,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    durability::*, storage::RelationalStorageUnavailableCause as Cause,
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    cold_recovery::{source_with_history, update_entity},
    durable_medium::TestDurableMedium,
    durable_publication::{durable_runtime, memory_runtime},
};
#[test]
fn cold_visibility_doors_preserve_storage_failure_and_never_advance_publication() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let original = source_with_history(execution, 0, false);
    for door in ReadDoor::ALL {
        let (source, counter) = copy_input(&original);
        let entity = original.entity;
        counter.damaged.store(true, Ordering::Relaxed);
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        door.prepare(&runtime);
        let before = runtime.history().latest_patch_stream_position();
        let failure = door.run(execution, &runtime, entity).unwrap_err();
        assert!(
            matches!(failure, RelationalExecutionDenialCause::StorageUnavailable(
            worth_relational::facade::storage::RelationalStorageUnavailable {
                subject: worth_relational::facade::storage::RelationalStorageSubject::Region(partition),
                cause: Cause::IntegrityMismatch,
            }) if partition == entity.partition_id),
            "{door:?}: {failure:?}"
        );
        let reads = counter.reads.load(Ordering::Relaxed);
        assert!(reads > 0);
        counter.damaged.store(false, Ordering::Relaxed);
        assert_eq!(
            door.run(execution, &runtime, entity).unwrap_err(),
            failure,
            "{door:?}"
        );
        assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
        assert_eq!(runtime.history().latest_patch_stream_position(), before);
        assert!(medium.offered().is_empty());
    }
}

#[test]
fn cold_and_warm_visibility_preserve_an_observation_across_a_later_commit() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let reference = memory_runtime();
    let expected_entity = create_entity(request, &reference, "member-before");
    let expected = reference
        .read_truth()
        .visible_entity_at_version(request, expected_entity, main_selected_version(&reference))
        .unwrap()
        .unwrap();
    let original = source_with_history(request, 0, false);
    let entity = original.entity;
    let (source, counter) = copy_input(&original);
    let medium = TestDurableMedium::new();
    let attachment = medium.attachment();
    for bytes in counter.commits.values() {
        let a = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&a),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    let cold = runtime
        .read_truth()
        .visible_entity_at_version(request, entity, main_selected_version(&runtime))
        .unwrap()
        .unwrap();
    assert_eq!(cold, expected);
    let reads = counter.reads.load(Ordering::Relaxed);
    let warm = runtime
        .read_truth()
        .visible_entity_at_version(request, entity, main_selected_version(&runtime))
        .unwrap()
        .unwrap();
    assert_eq!(warm, expected);
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    update_entity(request, &runtime, entity, "member-after");
    let retained = runtime
        .read_truth()
        .read_observation(request, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities(), [expected]);
}
