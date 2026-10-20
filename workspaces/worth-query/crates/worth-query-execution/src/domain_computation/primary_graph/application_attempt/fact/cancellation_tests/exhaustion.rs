//! Required source evidence cannot turn request exhaustion into inequality.
use super::*;
use std::{collections::BTreeMap, sync::Mutex};
use worth_execution::{
    ExecutionMap, ExecutionWorkCeiling, KeylessPartition, MapKernelFailure, MapKernelStop,
    MapOutcome, MapStop, WorkCeilingDenial,
};
use worth_foundational::PartitionIdentity;

#[test]
fn exhausted_native_fact_reads_preserve_the_request_stop() {
    let (healthy, results, stopped) = with_indexed_fact(
        |runtime, execution, snapshot, original, _, relation_kind| {
            let WorthQueryApplicationObservedFact::IndexedEntitySelection {
                candidates,
                entity_kind,
                locator,
                value,
                ..
            } = original
            else {
                unreachable!()
            };
            let entity = candidates[0];
            let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
            let revision = view
                .entity_field_revision(execution, entity, locator)
                .unwrap();
            let direction =
                worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming;
            let adjacency_revision = view
                .bounded_adjacency_structural_revision(
                    execution,
                    entity,
                    relation_kind,
                    direction,
                    32,
                )
                .unwrap()
                .revision();
            let endpoints = view
                .bounded_incoming_relations_of_kind(execution, entity, relation_kind, Some(32))
                .unwrap()
                .unwrap()
                .into_records()
                .into_iter()
                .map(|record| record.source)
                .collect::<Vec<_>>();
            let facts = [
            WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity },
            WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id: entity, locator: locator.clone(), native_revision: revision,
            },
            WorthQueryApplicationObservedFact::Field {
                entity_id: entity, kind: *entity_kind, locator: locator.clone(), value: value.clone(),
            },
            WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity, kind: *entity_kind, locator: locator.clone(),
            },
            WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind, anchor: entity, direction, native_revision: adjacency_revision,
                comparison_work_limit: 32,
                endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(
                    &endpoints, worth_execution::ExecutionAllocationPolicy::SystemAllocation, None,
                ).unwrap(),
            },
        ];
            let healthy = facts
                .iter()
                .zip([true, revision.is_some(), true, false, true])
                .map(|(fact, expected)| {
                    (
                        fact.remains_equal_in(runtime, execution, snapshot),
                        expected,
                    )
                })
                .collect::<Vec<_>>();
            let results = Mutex::new(Vec::new());
            let map = ExecutionMap::<_, u64>::from_keyless_partitions(BTreeMap::from([(
                PartitionIdentity::new(0),
                KeylessPartition {
                    value: (),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )]))
            .unwrap();
            let (outcome, _) = execution
                .run(ExecutionWorkCeiling::new(Some(0)), |_| {
                    map.run(
                        None,
                        |_, context| -> Result<u64, MapKernelFailure<MapKernelStop>> {
                            assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                            assert_eq!(
                                execution.consult(),
                                Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
                            );
                            for fact in &facts {
                                results
                                    .lock()
                                    .unwrap()
                                    .push(fact.remains_equal_in(runtime, execution, snapshot));
                            }
                            Ok(0)
                        },
                    )
                })
                .unwrap();
            let stopped = matches!(
                outcome,
                MapOutcome::Stopped {
                    reason: MapStop::WorkExhausted { .. },
                    ..
                }
            );
            (healthy, results.into_inner().unwrap(), stopped)
        },
    );
    assert!(stopped);
    for (index, (result, expected)) in healthy.into_iter().enumerate() {
        assert_eq!(result.unwrap(), expected, "healthy fact {index}");
    }
    assert_eq!(results.len(), 5, "all comparisons ran after exhaustion");
    for result in results {
        assert_eq!(result.unwrap_err().kind(), Kind::ExecutionRequest(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None, policy_ancestor: None,
                },
            ));
    }
}
