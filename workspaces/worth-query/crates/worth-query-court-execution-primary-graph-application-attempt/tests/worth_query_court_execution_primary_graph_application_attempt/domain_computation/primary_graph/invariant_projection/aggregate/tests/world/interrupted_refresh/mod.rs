//! An interrupted commit refresh must evict a previously materialized sum.
use crate::domain_computation::primary_graph::invariant_projection::aggregate::tests::world::AggregateWorld;
use std::collections::BTreeMap;
use worth_execution::ExecutionMap;
use worth_execution::ExecutionWorkCeiling;
use worth_execution::KeylessPartition;
use worth_execution::MapKernelFailure;
use worth_execution::MapKernelStop;
use worth_execution::MapOutcome;
use worth_execution::MapStop;
use worth_execution::WorkCeilingDenial;
use worth_foundational::facade::AspectValue;
use worth_foundational::PartitionIdentity;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn exhausted_affected_source_read_cannot_stamp_an_old_sum_current() {
    let world = AggregateWorld::values([7]);
    world
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let phase = world.fixture_phase(&custody);
            let healthy = custody.borrowed_request_for_test();
            assert_eq!(world.observe(&custody).result, Ok((7, 1)));
            assert_eq!(world.observe(&custody).work.aggregate_cache_hits(), 1);
            let source = world.source(&phase, "source-0");
            let locator = (*world
                .authority
                .layout_for_test())
                .field_locator_for_test("AggregateSource", "SourceFacts", "SourceAmount")
                .unwrap()
                .clone();
            (*world.authority.graph_for_test()).with_runtime_for_test(&healthy, |runtime| {
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
                let retained = (*(*world.authority.graph_for_test()).aggregate_projections_for_test()).lock().unwrap().clone();

                worth_query_execution::test_read_stops::every_consult_under(&healthy, |request| retained.clone().refresh_after_commit_for_test(request, runtime, &before, &committed.snapshot, committed.patch()), worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled, worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
                let map = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
                    PartitionIdentity::new(0),
                    KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )]))
                .unwrap();

                let refreshed = std::sync::atomic::AtomicBool::new(false);
                let (outcome, report) = healthy
                    .run(ExecutionWorkCeiling::new(Some(0)), |lease| {
                        map.run(
                            lease,
                            |_, context| -> Result<u64, MapKernelFailure<MapKernelStop>> {
                                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                                assert_eq!(
                                    healthy.consult(),
                                    Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                                );
                                let stopped = (*(*world
                                    .authority
                                    .graph_for_test())
                                    .aggregate_projections_for_test())
                                    .lock()
                                    .unwrap()
                                    .refresh_after_commit_for_test(
                                        &healthy,
                                        runtime,
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
                runtime.snapshots().release_snapshot(&before).unwrap();
                worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
                    runtime,
                    &committed.snapshot,
                );
            });
            assert_eq!(world.observe_on(&phase, world.main_branch()), Ok((11, 1)));
        })
        .expect("the aggregate refresh owns its host advancement");
}
