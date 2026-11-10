//! Replay reference outcomes through the public retention owner.
use super::replay_source::reopened;
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause,
};
#[test]
fn replay_pinning_preserves_sticky_mirror_refusal_without_publication() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let runtime = &fixture.runtime;
    let position = runtime.history().latest_patch_stream_position();
    let offered = fixture.medium.offered().len();
    // The retained historical root is already Full; corrupt every unread
    // region frame so the mirror's chosen serving carrier must refuse.
    fixture.source.damaged.store(true, Ordering::Relaxed);
    let expected =
        RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(fixture.entity.partition_id),
            cause: Cause::IntegrityMismatch,
        });
    let observed = runtime
        .history_authority()
        .retain_version_for_replay(execution, fixture.historical_version);
    assert_eq!(observed, Err(expected.into()));
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    assert!(reads > 0);
    fixture.source.damaged.store(false, Ordering::Relaxed);
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, fixture.historical_version),
        Err(expected.into())
    );
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        reads,
        "mirror damage stays sticky"
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(fixture.medium.offered().len(), offered);
}

#[test]
fn replay_retention_releases_original_references_without_cold_reads() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let runtime = &fixture.runtime;
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, fixture.historical_version),
        Ok(true)
    );
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, fixture.historical_version),
        Ok(true)
    );
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    fixture.source.damaged.store(true, Ordering::Relaxed);
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert!(!runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        reads,
        "release performs no cold loads"
    );
}
