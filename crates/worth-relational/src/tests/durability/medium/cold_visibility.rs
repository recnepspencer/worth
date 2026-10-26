//! Ordinary visibility doors must refuse cold damage before yielding evidence.
use super::*;
use crate::execution::RelationalExecutionDenialCause;

pub(super) use super::cold_read_door::ReadDoor;

#[test]
fn cold_visibility_doors_preserve_storage_failure_and_never_advance_publication() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in ReadDoor::ALL {
        let (source, counter, entity) = fixture(execution, false);
        let mut runtime = durable_runtime_with_test_schema();
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
        let before = runtime.history().latest_patch_stream_position();
        let failure = door.run(execution, &runtime, entity).unwrap_err();
        assert!(
            matches!(failure, RelationalExecutionDenialCause::StorageUnavailable(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(partition),
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
        assert!(attached_test_medium(&runtime).offered().is_empty());
    }
}

#[test]
fn cold_and_warm_visibility_preserve_an_observation_across_a_later_commit() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let reference = runtime_with_test_schema();
    let expected_entity = create_entity(execution, &reference, "member-before");
    let expected = reference
        .read_truth()
        .visible_entity_at_version(execution, expected_entity, reference.current_version_id())
        .unwrap()
        .unwrap();
    let (source, counter, entity) = fixture(execution, false);
    counter.damaged.store(false, Ordering::Relaxed);
    // Reopening a writer requires the medium and recovery source to describe
    // the same durable prefix. Publish the original artifacts, not synthetic
    // positions or a manually advanced frontier, into this test attachment.
    let attachment = TestDurableMedium::new().attachment();
    for bytes in counter.commits.values() {
        let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
        assert!(matches!(
            attachment.publish(&artifact),
            crate::durability::medium::RelationalDurablePublication::Committed
        ));
    }
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(attachment);
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
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    let observation = transaction.basis.observation();
    let cold = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, runtime.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(cold, expected);
    let reads = counter.reads.load(Ordering::Relaxed);
    let warm = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, runtime.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(warm, expected);
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
    update_entity_and_release_snapshot(execution, &runtime, entity, "member-after");
    let retained = runtime
        .read_truth()
        .read_observation(execution, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities, vec![expected]);
}
