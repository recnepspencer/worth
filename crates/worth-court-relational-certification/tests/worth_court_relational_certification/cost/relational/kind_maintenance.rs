//! Public SDK journey: one relation added to a growing, single-region model.
use crate::content::historical_kind::fixture::World;
use std::time::Instant;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    transactions::{CommitResult, CreateIntent, MutationIntent, RecordRef, WorkerIntentBatch},
};

#[test]
fn public_scale_small_edits_on_growing_model() {
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
    world.create(&execution, 7, "source");
    world.create(&execution, 7, "target");
    let mut relations = 0;
    for scale in [512, 1024, 2048] {
        let mut entities = WorkerIntentBatch::new("scale-targets");
        for index in world.entities.len()..scale + 11 {
            entities = entities.push(MutationIntent::Create(CreateIntent::Entity(
                World::entity_spec(7, &format!("target-{index}")),
            )));
        }
        let created = commit_batch(&world, &execution, entities);
        world.entities.extend(
            created
                .changed_records
                .iter()
                .filter_map(|record| match record {
                    RecordRef::Entity(id) => Some(*id),
                    RecordRef::Relation(_) => None,
                }),
        );
        world.release(&created);
        let mut batch = WorkerIntentBatch::new("scale-seed");
        for index in relations..scale {
            batch = batch.push(world.relation_intent(0, index + 1, 7, &format!("seed-{index}")));
        }
        let seeded = commit_batch(&world, &execution, batch);
        world.release(&seeded);
        relations = scale;

        let mut micros = Vec::new();
        for edit in 0..9 {
            let intent =
                world.relation_intent(0, scale + edit + 1, 7, &format!("edit-{scale}-{edit}"));
            let start = Instant::now();
            let committed = world.commit(&execution, intent).unwrap();
            micros.push(start.elapsed().as_micros());
            world.release(&committed);
            relations += 1;
        }
        micros.sort_unstable();
        eprintln!(
            "PUBLIC_COMMIT_SCALE relations={scale} edits=9 median_us={} min_us={} max_us={}",
            micros[4], micros[0], micros[8]
        );
    }
}

fn commit_batch(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, &world.runtime).unwrap()
}
#[test]
fn phase_observation_includes_nested_kind_work_without_changing_the_bill() {
    use worth_relational::facade::inspection::{
        observe_maintenance_work, MaintenanceWorkObservation,
    };
    fn run(observed: bool) -> (u64, MaintenanceWorkObservation) {
        let request = SerialRequest::from_policy(
            &ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(64 * 1024 * 1024),
                    Some(8_000_000),
                ),
            ),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&request);
        let mut world = World::new();
        world.create(&execution, 7, "left");
        world.create(&execution, 7, "right");
        let intent = world.relation_intent(0, 1, 7, "edge");
        let ((commit, observation), report) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                if observed {
                    observe_maintenance_work(32, || world.commit(&execution, intent))
                } else {
                    (
                        world.commit(&execution, intent),
                        MaintenanceWorkObservation::default(),
                    )
                }
            })
            .unwrap();
        let commit = commit.unwrap();
        assert!(!commit.changed_records.is_empty());
        world.release(&commit);
        (report.charged_work(), observation)
    }
    let (ordinary, _) = run(false);
    let (observed, observation) = run(true);
    assert_eq!(
        ordinary, observed,
        "inspection must preserve the request bill"
    );
    assert_eq!(observation.omitted_phases, 0);
    let work = |name| {
        observation
            .phases
            .iter()
            .filter(|row| row.phase == name)
            .map(|row| {
                assert!(row.accounted);
                row.charged_work
            })
            .sum::<u64>()
    };
    let kind = work("kind_commitment");
    assert!(
        kind > 0,
        "the real adopter must charge its projection/digest work"
    );
    assert!(
        work("root_admission") >= kind,
        "root admission includes region work"
    );
}
