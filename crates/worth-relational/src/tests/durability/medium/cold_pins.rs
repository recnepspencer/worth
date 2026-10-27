//! Replay pinning demands the mirror independently of its historical root.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, VersionId};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

struct PinFixture {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    historical_version: VersionId,
    entity: EntityId,
    source_state: crate::visibility::snapshot_states::SnapshotState,
}

fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> PinFixture {
    let producer = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &producer, "retained-pin-source");
    let (_, fork_basis) = producer
        .observe_fork_source(&crate::history::data::BranchId("main".to_owned()))
        .unwrap();
    producer
        .fork_branch(
            execution,
            crate::history::data::BranchId("pin-retained-source".to_owned()),
            fork_basis,
        )
        .unwrap();
    let (_, retained_basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    // The old root must actually remain retained across the producer's commits
    // and checkpoint; a catalog receipt alone does not retain its region cells.
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 2);
    drop(retained_basis);
    source.damaged.store(false, Ordering::Relaxed);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Input>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    let first = manifest.commits.first().unwrap();
    let mirror_history = manifest
        .mirror
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .history;
    let historical_history = manifest
        .roots
        .iter()
        .find(|root| root.commit_id == first.commit.commit_id)
        .unwrap()
        .regions
        .iter()
        .find(|region| region.partition_id == entity.partition_id)
        .unwrap()
        .history;
    assert_ne!(
        historical_history, mirror_history,
        "historical admission cannot witness the distinct mirror history"
    );
    let historical_version = first.commit.version_id;
    let attachment = TestDurableMedium::new().attachment();
    for bytes in source.commits.values() {
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
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no region member"
    );
    let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::capture_current(
        &runtime,
        &crate::history::data::BranchId("pin-retained-source".to_owned()),
        historical_version,
    )
    .unwrap()
    .expect("the recovered fork retains the exact old root");
    basis
        .root()
        .demand_all_full(execution, &runtime.services.symbols)
        .unwrap();
    let source_state = crate::visibility::snapshot_states::build_visibility_state(
        &runtime,
        basis,
        crate::snapshots::data::SnapshotId(0),
        crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
    );
    assert_eq!(
        source_state.pinned_entity_count, 1,
        "the exact source root has its real record"
    );
    assert!(
        matches!(
            crate::storage::overlay::PartitionAccess::probe_full_partition(
                &runtime.acquire_partition_edition(),
                entity.partition_id
            ),
            crate::storage::residency::PartitionProbe::Cold
        ),
        "successful old-root admission leaves the distinct mirror cold"
    );
    PinFixture {
        runtime,
        source,
        historical_version,
        entity,
        source_state,
    }
}

#[test]
fn replay_pinning_refuses_a_damaged_mirror_without_installing_retention() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let runtime = &fixture.runtime;
    let position = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(runtime).offered().len();
    assert!(
        runtime.visibility.replay_retention_versions().is_empty(),
        "fixture has no prior replay retention"
    );
    let reads_before = fixture.source.reads.load(Ordering::Relaxed);
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
    assert_eq!(
        observed,
        Err(expected.into()),
        "cache={:?}; reads={}",
        crate::visibility::cache_state::cached_historical_state_for_version(
            runtime,
            fixture.historical_version
        )
        .map(|state| (
            state.pinned_entity_count,
            state.pinned_relation_count,
            state.pinned_partitions.keys().copied().collect::<Vec<_>>()
        )),
        fixture.source.reads.load(Ordering::Relaxed) - reads_before
    );
    assert!(runtime.visibility.replay_retention_versions().is_empty());
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
    assert!(runtime.visibility.replay_retention_versions().is_empty());
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(attached_test_medium(runtime).offered().len(), offered);
}

fn replay_pin_count(fixture: &PinFixture) -> u32 {
    fixture
        .runtime
        .partitions
        .partition(fixture.entity.partition_id)
        .unwrap()
        .entity_arena
        .replay_pin_count(fixture.entity.slot_index())
        .unwrap()
}

#[test]
fn resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let position = fixture.runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&fixture.runtime).offered().len();
    let before = fixture.source.reads.load(Ordering::Relaxed);
    fixture.source.damaged.store(true, Ordering::Relaxed);
    let expected =
        RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(fixture.entity.partition_id),
            cause: Cause::IntegrityMismatch,
        });
    let demand = || {
        fixture
            .runtime
            .visibility_pins()
            .demand_replay_pins(execution, fixture.source_state.clone())
            .map(|_| ())
    };
    assert_eq!(demand(), Err(expected));
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    assert!(
        reads > before,
        "the fixture must actually cross the mirror's cold source"
    );
    fixture.source.damaged.store(false, Ordering::Relaxed);
    assert_eq!(demand(), Err(expected));
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
    assert!(fixture
        .runtime
        .visibility
        .replay_retention_versions()
        .is_empty());
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        position
    );
    assert_eq!(
        attached_test_medium(&fixture.runtime).offered().len(),
        offered
    );
}

#[test]
fn replay_retention_releases_from_owned_resident_custody_after_cache_clear() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let fixture = reopened(execution);
    let runtime = &fixture.runtime;
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, fixture.historical_version),
        Ok(true)
    );
    assert_eq!(replay_pin_count(&fixture), 1);
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, fixture.historical_version),
        Ok(true)
    );
    assert_eq!(
        replay_pin_count(&fixture),
        1,
        "additional retention references share physical pins"
    );
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    runtime.visibility.cache.clear();
    fixture.source.damaged.store(true, Ordering::Relaxed);
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert_eq!(replay_pin_count(&fixture), 1);
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert_eq!(replay_pin_count(&fixture), 0);
    assert!(!runtime
        .history_authority()
        .release_version_replay_retention(fixture.historical_version));
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        reads,
        "release performs no cold loads"
    );
}
