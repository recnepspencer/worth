use super::*;
use worth_execution::ExecutionRequest;

#[path = "replay_relations/cold_recovery.rs"]
mod cold_recovery;
#[path = "replay_relations/offered_medium.rs"]
mod offered_medium;
#[path = "replay_relations/resident_replay.rs"]
mod resident_replay;

type RetentionRequest<'host, 'scope> = ExecutionRequest<'host, 'scope>;

#[test]
fn retention_plan_reports_explicit_replay_pins_for_deleted_relations_until_released() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    resident_replay::assert_retention_lifecycle(execution);
}

#[derive(Debug)]
struct ColdPinSource {
    members: std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    >,
    damaged: std::sync::atomic::AtomicBool,
    reads: std::sync::atomic::AtomicUsize,
}
impl crate::durability::medium::RelationalDurableColdSource for ColdPinSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        crate::durability::medium::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as R;
        request.check_live().map_err(|c| R::Request(c.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &(dyn std::any::Any + Send + Sync),
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as R;
        use crate::storage::data::RelationalStorageUnavailableCause as Cause;
        request.check_live().map_err(|c| R::Request(c.into()))?;
        let bytes = self
            .members
            .get(&locator)
            .ok_or(R::Storage(Cause::MissingMember))?;
        if crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.as_ref()).is_err() {
            self.reads.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        if self.damaged.load(std::sync::atomic::Ordering::Relaxed)
            && bytes.starts_with(b"WORTH-REL-REG")
        {
            let mut corrupted = bytes.to_vec();
            corrupted[0] ^= 1;
            return Ok(
                worth_execution::ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
                    corrupted.into_boxed_slice(),
                )),
            );
        }
        Ok(bytes.clone())
    }
    fn retain_members(
        &self,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut bytes = self
            .members
            .get(&locator)
            .ok_or(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)?
            .to_vec();
        if self.damaged.load(std::sync::atomic::Ordering::Relaxed)
            && bytes.starts_with(b"WORTH-REL-REG")
        {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
}

// Fixture adaptation (C2.4): restores b736be6f47:cold_isolation.rs's private
// `resident_replay_pins` cold-partition refusal using ExternalMembers recovery
// into a Memory runtime and `VisibilitySnapshotBasis::from_applied_root`.
#[test]
fn replay_pin_of_a_cold_partition_is_typed_and_has_no_effect() {
    use crate::durability::data::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
    use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
    use crate::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailableCause as Cause,
    };
    use crate::storage::partition::DenseSlotBitSet;
    use crate::visibility::snapshot_states::{
        SnapshotState, SnapshotStateBasis, VisibilitySnapshotBasis,
    };
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let producer = runtime_with_test_schema();
    let entity = create_entity_in_partition(execution, &producer, "cold-pin-seed", PartitionId(1));
    let native = producer
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
    let (recovery, source) = cold_recovery::cold_pin_recovery(native);
    let mut cold = runtime_with_test_schema();
    let plan = cold
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    cold.durability_recovery().recover(execution, plan).unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    let branch = BranchId("main".into());
    let root = cold.history.branch_cell(&branch).unwrap().root().unwrap();
    let mut slots = DenseSlotBitSet::with_capacity(entity.slot_index() + 1);
    slots.set(entity.slot_index(), true);
    let version = cold.current_version_id();
    let state = SnapshotState {
        handle: SnapshotHandle {
            runtime_instance_id: cold.runtime_instance_id(),
            branch_id: branch.clone(),
            snapshot_id: SnapshotId(1),
            version_id: version,
            read_policy: SnapshotReadPolicy::ImmutablePinned,
        },
        basis: SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_applied_root(
            branch, version, root,
        )),
        pinned_partitions: std::collections::BTreeMap::from([(
            PartitionId(1),
            crate::storage::overlay::SnapshotPartitionPins {
                entity_slots: slots,
                relation_slots: DenseSlotBitSet::with_capacity(0),
                retained_relation_slots: DenseSlotBitSet::with_capacity(0),
            },
        )]),
        pinned_entity_count: 1,
        pinned_relation_count: 0,
    };
    let failure = cold
        .visibility_pins()
        .resident_replay_pins(state.clone())
        .unwrap_err();
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(PartitionId(1))
    );
    assert_eq!(failure.cause, Cause::SourceUnavailable);
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    create_entity_in_partition(execution, &cold, "warm-pin", PartitionId(1));
    assert_eq!(
        cold.visibility_pins()
            .resident_replay_pins(state.clone())
            .unwrap_err()
            .cause,
        Cause::SourceUnavailable
    );
    cold.acquire_partition_edition()
        .demand_all(execution, &cold.services.symbols)
        .unwrap();
    let fence = state.handle.version_id;
    let leases = cold.visibility_pins().resident_replay_pins(state).unwrap();
    assert_eq!(cold.entity_replay_pin_count(entity), Some(0));
    cold.visibility_pins().pin_replay_state(&leases, fence);
    assert_eq!(cold.entity_replay_pin_count(entity), Some(1));
    cold.visibility_pins().unpin_replay_state(&leases, fence);
    assert_eq!(cold.entity_replay_pin_count(entity), Some(0));
}

// Fixture adaptation (C2.4): restores b736be6f47/81d41f5575:cold_pins.rs's
// private `demand_replay_pins` mirror-damage refusal and `visibility.cache.clear()`
// release window over ExternalMembers recovery into a Memory runtime.
#[test]
fn resident_pin_custody_and_cache_clear_preserve_owned_mirror_residency() {
    use crate::durability::data::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
    use crate::execution::RelationalExecutionDenialCause;
    use crate::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    };
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let reopened = || {
        let producer = runtime_with_test_schema();
        let entity = create_entity(execution, &producer, "pin-before");
        let historical_version = producer.current_version_id();
        let fork_branch = BranchId("pin-retained-source".into());
        let (_, fork_basis) = producer
            .observe_fork_source(&BranchId("main".into()))
            .unwrap();
        producer
            .fork_branch(execution, fork_branch.clone(), fork_basis)
            .unwrap();
        update_entity_and_release_snapshot(execution, &producer, entity, "pin-after");
        let native = producer
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
        let (recovery, source) = cold_recovery::cold_pin_recovery(native);
        let offered = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut runtime = RelationalRuntimeApi::builder()
            .schema_registry(test_schema_registry())
            .build_durable(Box::new(offered_medium::OfferedSpy(
                offered.clone(),
                source.clone(),
            )))
            .unwrap();
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
        let basis = runtime
            .capture_current_visibility_basis(&fork_branch, historical_version)
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
        (
            runtime,
            source,
            offered,
            historical_version,
            entity,
            source_state,
        )
    };
    let (runtime, source, offered, _version, entity, source_state) = reopened();
    let frontier = runtime.history().latest_patch_stream_position();
    let before = source.reads.load(Ordering::Relaxed);
    source.damaged.store(true, Ordering::Relaxed);
    let expected =
        RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(entity.partition_id),
            cause: Cause::IntegrityMismatch,
        });
    assert_eq!(
        runtime
            .visibility_pins()
            .demand_replay_pins(execution, source_state.clone())
            .map(|_| ()),
        Err(expected.clone())
    );
    let reads = source.reads.load(Ordering::Relaxed);
    assert!(reads > before);
    source.damaged.store(false, Ordering::Relaxed);
    assert_eq!(
        runtime
            .visibility_pins()
            .demand_replay_pins(execution, source_state)
            .map(|_| ()),
        Err(expected)
    );
    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
    assert!(runtime.visibility.replay_retention_versions().is_empty());
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert!(
        offered.lock().unwrap().is_empty(),
        "refused replay pinning must offer no durable artifacts"
    );
    let (runtime, source, _offered, version, entity, _) = reopened();
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, version),
        Ok(true)
    );
    assert_eq!(runtime.entity_replay_pin_count(entity), Some(1));
    assert_eq!(
        runtime
            .history_authority()
            .retain_version_for_replay(execution, version),
        Ok(true)
    );
    assert_eq!(runtime.entity_replay_pin_count(entity), Some(1));
    let reads = source.reads.load(Ordering::Relaxed);
    runtime.visibility.cache.clear();
    source.damaged.store(true, Ordering::Relaxed);
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(version));
    assert_eq!(runtime.entity_replay_pin_count(entity), Some(1));
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(version));
    assert_eq!(runtime.entity_replay_pin_count(entity), Some(0));
    assert!(!runtime
        .history_authority()
        .release_version_replay_retention(version));
    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
}
