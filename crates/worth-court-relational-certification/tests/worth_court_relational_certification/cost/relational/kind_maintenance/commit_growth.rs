//! Real SDK commits guard the production phase, not only the delta kernel.
use super::{commit_batch, World};
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, RelationId},
    inspection::{observe_history_prefix_traversal, observe_maintenance_work},
    transactions::{
        CreateIntent, EntityReference, MutationIntent, RecordRef, RelationMutationIntent,
        UpdateRelationEndpointsIntent, WorkerIntentBatch,
    },
};
use worth_testkit_relational::maintenance::assert_change_bounded;

pub(super) fn assert_public_growth() {
    let model = (commit_work(512, 0), commit_work(1024, 0));
    let history = (commit_work(32, 1024), commit_work(32, 2048));
    eprintln!(
        "SDK root admission charged work: model {}->{}; history {}->{}",
        model.0 .0, model.1 .0, history.0 .0, history.1 .0
    );
    assert_change_bounded("SDK root admission", "model", model.0 .0, model.1 .0);
    assert_change_bounded("SDK root admission", "history", history.0 .0, history.1 .0);
    eprintln!(
        "SDK prefix traversal: model {}->{}; history {}->{}",
        model.0 .1, model.1 .1, history.0 .1, history.1 .1
    );
    assert_change_bounded(
        "SDK prefix traversal",
        "model",
        model.0 .1.max(1),
        model.1 .1.max(1),
    );
    assert_change_bounded(
        "SDK prefix traversal",
        "history",
        history.0 .1.max(1),
        history.1 .1.max(1),
    );
}

fn commit_work(model: usize, history: usize) -> (u64, u64) {
    let request = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1024 * 1024 * 1024),
                Some(1_000_000_000),
            ),
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let mut world = World::new();
    let mut batch = WorkerIntentBatch::new("growth-entities");
    for index in 0..model + 3 {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(
            World::entity_spec(7, &format!("growth-{index}")),
        )));
    }
    let seeded = commit_batch(&world, &execution, batch);
    world.entities.extend(
        seeded
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(id) => Some(*id),
                RecordRef::Relation(_) => None,
            }),
    );
    world.release(&seeded);
    let mut batch = WorkerIntentBatch::new("growth-relations");
    for index in 0..model {
        batch = batch.push(world.relation_intent(0, index + 1, 7, &format!("edge-{index}")));
    }
    let seeded = commit_batch(&world, &execution, batch);
    let relation = seeded
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Relation(id) => Some(*id),
            RecordRef::Entity(_) => None,
        })
        .unwrap();
    world.release(&seeded);
    for step in 0..history {
        let target = if step % 2 == 0 { model + 1 } else { 1 };
        let commit = world
            .commit(&execution, retarget(&world, relation, target))
            .unwrap();
        world.release(&commit);
    }
    // Both history scales end at the same live endpoint. The fixed edit uses
    // one fresh endpoint and changes one existing relation, never the roster.
    let ((commit, observation), prefixes) = observe_history_prefix_traversal(|| {
        observe_maintenance_work(32, || {
            world.commit(&execution, retarget(&world, relation, model + 2))
        })
    });
    world.release(&commit.unwrap());
    assert_eq!(
        observation.omitted_phases, 0,
        "phase evidence must be complete"
    );
    let phases: Vec<_> = observation
        .phases
        .iter()
        .filter(|row| row.phase == "root_admission")
        .collect();
    assert!(
        !phases.is_empty(),
        "the actual public commit must enter root admission"
    );
    let charged_work = phases
        .iter()
        .map(|row| {
            assert!(
                row.accounted,
                "growth must use real original-request charges"
            );
            row.charged_work
        })
        .sum();
    assert!(charged_work > 0, "root admission must charge its real work");
    (charged_work, prefixes)
}

fn retarget(world: &World, relation: RelationId, target: usize) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: relation,
            kind_id: KindId(2),
            source: EntityReference::Existing(world.entities[0]),
            target: EntityReference::Existing(world.entities[target]),
        },
    ))
}
