//! Relation-bearing cold roots must preserve the demanded region's isolation.
use super::*;
use crate::identity::data::{EntityId, PartitionId};
use std::collections::BTreeMap;

struct ColdRoot {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    entities: Vec<EntityId>,
}

fn recover_relations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    missing_history: Option<PartitionId>,
) -> ColdRoot {
    recover_shape(execution, missing_history, 16, false)
}

fn recover_shape(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    missing_history: Option<PartitionId>,
    count: u32,
    ring: bool,
) -> ColdRoot {
    let runtime = durable_runtime_with_test_schema();
    let mut entities = Vec::new();
    let mut targets = Vec::new();
    for id in 1..=count {
        let partition = PartitionId(id);
        let a = create_entity_in_partition(execution, &runtime, "source", partition);
        let b = create_entity_in_partition(execution, &runtime, "target", partition);
        entities.push(a);
        targets.push(b);
    }
    for (index, &a) in entities.iter().enumerate() {
        let b = if ring {
            entities[(index + 1) % entities.len()]
        } else {
            targets[index]
        };
        create_relation_in_partition(execution, &runtime, a, b, "edge", a.partition_id);
    }
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let input: Input = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let missing = missing_history.map(|id| {
        input.manifest.roots[0]
            .regions
            .iter()
            .find(|leaf| leaf.partition_id == id)
            .unwrap()
            .history
    });
    let artifacts = attached_test_medium(&runtime).offered();
    let source = Arc::new(MemberSource {
        members: native
            .members()
            .iter()
            .filter_map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } if Some(*locator) != missing => {
                    Some((*locator, bytes.clone()))
                }
                RelationalCheckpointMember::New { .. } => None,
                _ => panic!("first capture owns all members"),
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|entry| (entry.position(), entry.as_bytes().to_vec()))
            .collect(),
        damaged: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
        member_reads: std::sync::Mutex::new(Vec::new()),
        commit_reads: std::sync::Mutex::new(Vec::new()),
        commits_damaged: AtomicBool::new(false),
        failed_member: std::sync::Mutex::new(None),
        failures: std::sync::Mutex::new(Default::default()),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut receiving = durable_runtime_with_test_schema();
    bind_durable_medium(&mut receiving, runtime.durability.detach_medium().unwrap());
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    ColdRoot {
        runtime: receiving,
        source,
        entities,
    }
}

#[test]
fn a_relation_bearing_cold_commit_loads_only_its_partition() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_relations(execution, None);
    create_entity_in_partition(execution, &cold.runtime, "new", PartitionId(1));
    let reads = cold.source.reads.load(Ordering::Relaxed);
    assert!(reads < 12, "one-partition commit loaded {reads} members");
}

#[test]
fn missing_history_damages_only_its_own_region() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_relations(execution, Some(PartitionId(5)));
    // Exercise the damaged cell first, then prove unrelated cells remain usable.
    let root = cold
        .runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let first = root
        .demand_regions_full(execution, &cold.runtime.services.symbols, [PartitionId(5)])
        .unwrap_err();
    assert!(
        matches!(&first, crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(f)
        if f.subject == crate::storage::data::RelationalStorageSubject::Region(PartitionId(5)))
    );
    let reads = cold.source.reads.load(Ordering::Relaxed);
    assert_eq!(
        root.demand_regions_full(execution, &cold.runtime.services.symbols, [PartitionId(5)])
            .unwrap_err(),
        first
    );
    assert_eq!(cold.source.reads.load(Ordering::Relaxed), reads);
    for entity in cold
        .entities
        .iter()
        .filter(|entity| entity.partition_id != PartitionId(5))
    {
        let mut reader = test_owner_begin_transaction_for_main(&cold.runtime);
        assert!(reader
            .read_entity(execution, *entity)
            .unwrap()
            .effective()
            .is_some());
        create_entity_in_partition(execution, &cold.runtime, "healthy", entity.partition_id);
    }
}

#[test]
fn fork_of_a_cold_runtime_shares_cells_and_both_copies_can_read_and_write() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_relations(execution, None);
    let fork = cold.runtime.fork().unwrap();
    assert_eq!(
        cold.source.reads.load(Ordering::Relaxed),
        0,
        "fork must perform no I/O"
    );
    for (runtime, id, name) in [(&cold.runtime, 1, "origin"), (&fork, 2, "fork")] {
        let mut reader = test_owner_begin_transaction_for_main(runtime);
        assert!(reader
            .read_entity(execution, cold.entities[id - 1])
            .unwrap()
            .effective()
            .is_some());
        create_entity_in_partition(execution, runtime, name, PartitionId(id as u32));
        update_entity_and_release_snapshot(execution, runtime, cold.entities[id - 1], name);
    }
    for runtime in [&cold.runtime, &fork] {
        for entity in &cold.entities {
            let mut reader = test_owner_begin_transaction_for_main(runtime);
            assert!(reader
                .read_entity(execution, *entity)
                .unwrap()
                .effective()
                .is_some());
        }
    }
}

#[path = "cold_fork_edits.rs"]
mod fork_edits;

#[test]
fn replay_pin_of_a_cold_partition_is_typed_and_has_no_effect() {
    use crate::snapshots::data::{SnapshotHandle, SnapshotId, SnapshotReadPolicy};
    use crate::storage::partition::DenseSlotBitSet;
    use crate::visibility::snapshot_states::{
        SnapshotState, SnapshotStateBasis, VisibilitySnapshotBasis,
    };
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_shape(execution, None, 4, true);
    let branch = crate::history::data::BranchId("main".into());
    let root = cold
        .runtime
        .history
        .branch_cell(&branch)
        .unwrap()
        .root()
        .unwrap();
    let mut slots = DenseSlotBitSet::with_capacity(1);
    slots.set(cold.entities[0].slot_index(), true);
    // The pin set names an owner-issued committed record. Snapshot registration
    // is irrelevant to the pin authority's residency precondition.
    let state = SnapshotState {
        handle: SnapshotHandle {
            runtime_instance_id: cold.runtime.runtime_instance_id(),
            branch_id: branch.clone(),
            snapshot_id: SnapshotId(1),
            version_id: cold.runtime.current_version_id(),
            read_policy: SnapshotReadPolicy::ImmutablePinned,
        },
        basis: SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_prepared_root(branch, root)),
        pinned_partitions: BTreeMap::from([(
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
        .runtime
        .visibility_pins()
        .resident_replay_pins(state.clone())
        .unwrap_err();
    assert_eq!(
        failure.subject,
        crate::storage::data::RelationalStorageSubject::Region(PartitionId(1))
    );
    assert_eq!(failure.cause, Cause::SourceUnavailable);
    assert_eq!(cold.source.reads.load(Ordering::Relaxed), 0);
    create_entity_in_partition(execution, &cold.runtime, "warm-pin", PartitionId(1));
    // A current write admits only Head. Pin mutation needs the explicit Full witness.
    assert_eq!(
        cold.runtime
            .visibility_pins()
            .resident_replay_pins(state.clone())
            .unwrap_err()
            .cause,
        Cause::SourceUnavailable
    );
    cold.runtime
        .acquire_partition_edition()
        .demand_all(execution, &cold.runtime.services.symbols)
        .unwrap();
    let leases = cold
        .runtime
        .visibility_pins()
        .resident_replay_pins(state.clone())
        .unwrap();
    let fence = cold
        .runtime
        .visibility
        .retention_fence_version(cold.runtime.current_version_id());
    cold.runtime
        .visibility_pins()
        .pin_replay_state(&leases, fence);
    let other_holder = cold
        .runtime
        .visibility_pins()
        .resident_replay_pins(state.clone())
        .unwrap();
    cold.runtime
        .visibility_pins()
        .pin_replay_state(&other_holder, fence);
    let pin_count = || {
        cold.runtime
            .partitions
            .acquire()
            .partition(PartitionId(1))
            .unwrap()
            .entity_arena
            .replay_pin_count(cold.entities[0].slot_index())
    };
    assert_eq!(pin_count(), Some(2));
    cold.runtime
        .visibility_pins()
        .unpin_replay_state(&leases, fence);
    assert_eq!(pin_count(), Some(1));
    cold.runtime
        .visibility_pins()
        .unpin_replay_state(&other_holder, fence);
    assert_eq!(pin_count(), Some(0));
}
