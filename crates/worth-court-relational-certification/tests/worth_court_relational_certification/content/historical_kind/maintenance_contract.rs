//! Real canonical preparation carries one change into SDK maintenance dispatch.
use super::{fixture::World, funding::request};
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    storage::maintenance::{
        apply_change, ChangeWork, CommitChange, DigestIndex, Maintained, MaintenanceConsistency,
        MaintenanceRefusal, MaintenanceRoute, ReconstructionLane, ReconstructionWork,
    },
    transactions::WorkerIntentBatch,
};

#[test]
fn nested_maintenance_preserves_the_original_request_stop() {
    let request = request(1);
    let execution = ExecutionRequest::serial(&request);
    let (result, _) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            ChangeWork::run(&execution, |_| {
                ChangeWork::run(&execution, |inner| inner.prepare(2, 0))
            })
        })
        .unwrap();
    assert!(
        matches!(
        result,
        Err(MaintenanceRefusal::Execution(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted
        ))
    ),
        "nested maintenance must retain the original stop: {result:?}"
    );
}

#[test]
fn canonical_preparation_shares_one_sealed_change_and_untouched_state() {
    let request = request(8_000_000);
    let execution = ExecutionRequest::serial(&request);
    let mut world = World::new();
    world.create(&execution, 7, "left");
    world.create(&execution, 7, "right");
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("shared-change").push(world.relation_intent(0, 1, 7, "edge")),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    let change = candidate
        .commit_change()
        .expect("canonical preparation carries its journal");
    let again = candidate.commit_change().unwrap();
    assert!(
        Arc::ptr_eq(&change, &again),
        "consumers share one projection"
    );
    let native_records = candidate
        .with_prepared_changed_records(|records| records.to_vec())
        .unwrap();
    assert_eq!(change.created(), native_records);
    let records = change.partition(PartitionId(7)).unwrap().records();
    for record in records {
        assert!(record.before.is_none());
        assert_eq!(record.after.as_ref().unwrap().kind, KindId(2));
    }
    let native_touches = candidate
        .with_prepared_descriptive_touches(|touches| touches.exact_touches().unwrap().to_vec())
        .unwrap();
    assert_eq!(change.touches().unwrap(), native_touches);
    assert!(!change.schema_changed());
    assert!(change
        .routes()
        .contains(&MaintenanceRoute::RelationKind(KindId(2))));

    let previous = Arc::new(DigestIndex::default());
    let mut untouched = Untouched {
        routes: BTreeSet::new(),
    };
    let carried = ChangeWork::run(&execution, |work| {
        apply_change(&mut untouched, Some(&previous), &change, work)
    })
    .unwrap();
    assert!(Arc::ptr_eq(&previous, &carried));
    let refused = ChangeWork::run(&execution, |work| {
        apply_change(&mut untouched, None, &change, work)
    });
    assert!(matches!(
        refused,
        Err(MaintenanceRefusal::MissingPredecessor)
    ));
    world
        .runtime
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .unwrap();
}

/// This routing spy must never be entered for an unrelated canonical change.
struct Untouched {
    routes: BTreeSet<MaintenanceRoute>,
}
impl Maintained for Untouched {
    type State = DigestIndex;
    type Retirement = std::convert::Infallible;
    type Error = MaintenanceRefusal;
    fn routes(&self) -> &BTreeSet<MaintenanceRoute> {
        &self.routes
    }
    fn consistency(&self) -> MaintenanceConsistency {
        MaintenanceConsistency::Synchronous
    }
    fn apply(
        &mut self,
        _: &Arc<Self::State>,
        _: &CommitChange,
        _: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        panic!("an unrelated owner must not be invoked")
    }
    fn cold(
        &mut self,
        _: &ReconstructionLane<'_>,
        _: &mut ReconstructionWork<'_, '_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        panic!("ordinary dispatch must not reconstruct")
    }
    fn retire(
        &mut self,
        _: &Arc<Self::State>,
        horizon: &Self::Retirement,
        _: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self::State>, Self::Error> {
        match *horizon {}
    }
}
