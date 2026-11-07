//! Count-only public reads consume the selected root's authenticated facts.
use super::cold_checkpoint::{reopen, ColdCheckpointSource};
use crate::content::historical_kind::fixture::World;
use std::sync::atomic::Ordering;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::*, identity::KindId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
    transactions::*,
};
use worth_testkit_relational::maintenance::assert_change_bounded;

fn snapshot(runtime: &RelationalRuntime) -> SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}
fn equals_rows(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    handle: &SnapshotHandle,
) {
    let counts = runtime
        .read_truth()
        .inspect_snapshot(request, handle)
        .unwrap()
        .unwrap();
    let rows = runtime
        .read_truth()
        .read_snapshot(handle, request)
        .unwrap()
        .unwrap();
    assert_eq!(counts.entity_count, rows.entities().len());
    assert_eq!(counts.relation_count, rows.relations().len());
}

#[test]
fn selected_counts_equal_full_rows_after_mixed_edits_and_branch_movement() {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    world.create(&request, 7, "selected");
    let retained = snapshot(&world.runtime);
    for step in 0..12 {
        world.create(&request, 7 + step % 3, "next");
        if step % 3 == 0 {
            world.relate(&request, 0, world.entities.len() - 1, 9, "edge");
        }
        if step % 3 == 1 {
            world.replace(&request, 1, 8, "replacement");
        }
        if step % 3 == 2 {
            world.delete_entity(&request, world.entities.len() - 1);
        }
        let current = snapshot(&world.runtime);
        equals_rows(&world.runtime, &request, &current);
        equals_rows(&world.runtime, &request, &retained);
        world
            .runtime
            .snapshots()
            .release_snapshot(&current)
            .unwrap();
    }
    world
        .runtime
        .snapshots()
        .release_snapshot(&retained)
        .unwrap();
}

fn count_read_bytes(regions: u32, history: usize) -> u64 {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    for label in ["selected", "target-a", "target-b"] {
        world.create(&request, 7, label);
    }
    for region in 0..regions {
        world.create(&request, 100 + region, "unrelated");
    }
    world.relate(&request, 0, 1, 7, "selected edge");
    for step in 0..history {
        let committed = world
            .commit(
                &request,
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                    UpdateRelationEndpointsIntent {
                        relation_id: world.relations[0],
                        kind_id: KindId(2),
                        source: EntityReference::Existing(world.entities[0]),
                        target: EntityReference::Existing(world.entities[1 + step % 2]),
                    },
                )),
            )
            .unwrap();
        world.release(&committed);
    }
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = ColdCheckpointSource::capture(&world.runtime, &request, &checkpoint);
    let runtime = reopen(&request, &checkpoint, source.clone());
    let handle = snapshot(&runtime);
    let before = source.bytes.load(Ordering::SeqCst);
    let counts = runtime
        .read_truth()
        .inspect_snapshot(&request, &handle)
        .unwrap()
        .unwrap();
    assert_eq!(counts.entity_count, world.entities.len());
    assert_eq!(counts.relation_count, world.relations.len());
    let work = source.bytes.load(Ordering::SeqCst) - before;
    // The oracle is a whole-result read after the measured count-only operation.
    equals_rows(&runtime, &request, &handle);
    runtime.snapshots().release_snapshot(&handle).unwrap();
    work
}
#[test]
fn count_only_read_does_not_grow_with_model() {
    let model = [count_read_bytes(16, 8), count_read_bytes(32, 8)];
    println!(
        "SDK snapshot count cold bytes: model {}->{}",
        model[0], model[1]
    );
    assert_change_bounded("snapshot counts", "model", model[0], model[1]);
}
#[test]
fn count_only_read_does_not_grow_with_history() {
    let history = [count_read_bytes(4, 128), count_read_bytes(4, 256)];
    println!(
        "SDK snapshot count cold bytes: history {}->{}",
        history[0], history[1]
    );
    assert_change_bounded("snapshot counts", "history", history[0], history[1]);
}
