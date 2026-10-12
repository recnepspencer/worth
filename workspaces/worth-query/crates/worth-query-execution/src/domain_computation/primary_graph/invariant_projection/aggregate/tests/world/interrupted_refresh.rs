//! An interrupted commit refresh must evict a previously materialized sum.
use super::{AggregateWorld, AspectFieldPatch, AspectValue, BTreeMap};
use worth_execution::{
    ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, KeylessPartition, MapKernelFailure,
    MapKernelStop, MapOutcome, MapStop, WorkCeilingDenial,
};
use worth_foundational::PartitionIdentity;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

#[test]
fn exhausted_affected_source_read_cannot_stamp_an_old_sum_current() {
    let world = AggregateWorld::values([7]);
    world
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let phase = world.fixture_phase(&custody);
            let healthy_host = crate::test_request_policy::serial_request();
            let healthy = ExecutionRequest::serial(&healthy_host);
            assert_eq!(world.observe(&custody).result, Ok((7, 1)));
            assert_eq!(world.observe(&custody).work.aggregate_cache_hits(), 1);
            let source = world.source(&phase, "source-0");
            let locator = world
                .authority
                .layout
                .field_locator("AggregateSource", "SourceFacts", "SourceAmount")
                .unwrap()
                .clone();
            world.authority.graph.with_runtime_mut(&healthy, |runtime| {
                let identity = runtime.main_branch_identity();
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let before = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let mut transaction = runtime
                    .begin_branch_transaction(
                        &basis,
                        worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .unwrap();
                transaction
                    .push_batch(
                        WorkerIntentBatch::new("aggregate-interrupted-refresh").push(
                            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                                UpdateEntityFieldsIntent {
                                    entity_id: source,
                                    fields: AspectFieldPatch::from(BTreeMap::from([(
                                        locator,
                                        AspectValue::Int64(11),
                                    )])),
                                },
                            )),
                        ),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                let committed = transaction.commit(&healthy, runtime).unwrap();
                let retained = world.authority.graph.aggregate_projections.lock().unwrap().clone();
                let runtime_cell = std::cell::RefCell::new(&mut *runtime);
                crate::test_read_stops::every_consult(|request| retained.clone().refresh_after_commit(request, &mut runtime_cell.borrow_mut(), &before, &committed.snapshot, committed.patch()), worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled, worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
                drop(runtime_cell);
                let map = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
                    PartitionIdentity::new(0),
                    KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )]))
                .unwrap();
                let refresh_runtime = std::sync::Mutex::new(&mut *runtime);
                let refreshed = std::sync::atomic::AtomicBool::new(false);
                let (outcome, report) = healthy
                    .run(ExecutionWorkCeiling::new(0), |_| {
                        map.run(
                            None,
                            |_, context| -> Result<u64, MapKernelFailure<MapKernelStop>> {
                                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                                assert_eq!(
                                    healthy.consult(),
                                    Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                                );
                                let stopped = world
                                    .authority
                                    .graph
                                    .aggregate_projections
                                    .lock()
                                    .unwrap()
                                    .refresh_after_commit(
                                        &healthy,
                                        &mut refresh_runtime.lock().unwrap(),
                                        &before,
                                        &committed.snapshot,
                                        committed.patch(),
                                    );
                                assert_eq!(stopped, Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted));
                                refreshed.store(true, std::sync::atomic::Ordering::SeqCst);
                                Ok(0)
                            },
                        )
                    })
                    .unwrap();
                assert!(matches!(
                    outcome,
                    MapOutcome::Stopped {
                        reason: MapStop::WorkExhausted { .. },
                        ..
                    }
                ));
                assert!(refreshed.load(std::sync::atomic::Ordering::SeqCst));
                assert_eq!(report.charged_work(), 0);
                drop(refresh_runtime);
                runtime.snapshots().release_snapshot(&before).unwrap();
                crate::relational_snapshot_release::release_query_snapshot(
                    runtime,
                    &committed.snapshot,
                );
            });
            assert_eq!(world.observe_on(&phase, world.main_branch()), Ok((11, 1)));
        })
        .expect("the aggregate refresh owns its host advancement");
}
