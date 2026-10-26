use super::*;
use crate::facade::{config::CascadeDeletePolicy, identity::PartitionId};
use crate::tests::support::*;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

fn scope(roots: &[EntityId], direction: RelationDirection) -> PinnedStructuralWalkScope<'_> {
    PinnedStructuralWalkScope::new(roots, KindId(1), KindId(2), direction)
}

#[test]
fn complete_walk_is_global_canonical_and_keeps_physical_cycle_edges() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let ids: Vec<_> = (0..7)
        .map(|i| {
            create_entity_in_partition(&request, &runtime, &format!("walk-{i}"), PartitionId(7))
        })
        .collect();
    let pairs = [(0, 0), (0, 1), (0, 2), (1, 3), (2, 3), (3, 0), (3, 4)];
    let edges: Vec<_> = pairs
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            create_relation_in_partition(
                &request,
                &runtime,
                ids[a],
                ids[b],
                &format!("edge-{i}"),
                PartitionId(7),
            )
        })
        .collect();
    let pin = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &request)
        .unwrap();
    let walk = pin
        .complete_relation_walk(&request, scope(&ids[..1], RelationDirection::Forward))
        .unwrap();
    assert_eq!(
        walk.visits()
            .iter()
            .map(|v| (v.entity.entity_id, v.distance))
            .collect::<Vec<_>>(),
        vec![
            (ids[0], 0),
            (ids[1], 1),
            (ids[2], 1),
            (ids[3], 2),
            (ids[4], 3)
        ]
    );
    assert_eq!(
        walk.relations()
            .iter()
            .map(|r| r.relation_id)
            .collect::<Vec<_>>(),
        edges
    );
    assert_eq!(
        walk.reads(),
        PinnedStructuralWalkReads {
            adjacency_lists: 5,
            relation_records: 7,
            entity_records: 5
        }
    );
    let reverse = pin
        .complete_relation_walk(&request, scope(&ids[4..5], RelationDirection::Reverse))
        .unwrap();
    assert_eq!(
        reverse
            .visits()
            .iter()
            .map(|v| (v.entity.entity_id, v.distance))
            .collect::<Vec<_>>(),
        vec![
            (ids[4], 0),
            (ids[3], 1),
            (ids[1], 2),
            (ids[2], 2),
            (ids[0], 3)
        ]
    );
    assert_eq!(reverse.relations(), walk.relations());
    let roots = [ids[6], ids[3], ids[0], ids[2], ids[1], ids[5], ids[3]];
    let all = pin
        .complete_relation_walk(&request, scope(&roots, RelationDirection::Forward))
        .unwrap();
    let shuffled: Vec<_> = roots.into_iter().rev().collect();
    let twin = pin
        .complete_relation_walk(&request, scope(&shuffled, RelationDirection::Forward))
        .unwrap();
    assert_eq!(all.visits(), twin.visits());
    assert_eq!(all.relations(), twin.relations());
    assert_eq!(all.visits().len(), 7);
    assert_eq!(all.visits().last().unwrap().distance, 1);
    assert_eq!(all.reads().entity_records, 7);
    assert_eq!(all.reads().relation_records, 7);
}

#[test]
fn pinned_walk_survives_owner_drop_and_excludes_later_entities() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&request, &runtime, "b", PartitionId(7));
    create_relation_in_partition(&request, &runtime, a, b, "ab", PartitionId(7));
    let before = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &request)
        .unwrap();
    let c = create_entity_in_partition(&request, &runtime, "c", PartitionId(7));
    create_relation_in_partition(&request, &runtime, b, c, "bc", PartitionId(7));
    let after = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &request)
        .unwrap();
    drop(runtime);
    let old = before
        .complete_relation_walk(&request, scope(&[a], RelationDirection::Forward))
        .unwrap();
    let new = after
        .complete_relation_walk(&request, scope(&[a], RelationDirection::Forward))
        .unwrap();
    assert_eq!(old.visits().len(), 2);
    assert_eq!(new.visits().len(), 3);
}

#[test]
fn original_work_cancellation_and_retained_memory_survive_handoff() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&request, &runtime, "b", PartitionId(7));
    create_relation_in_partition(&request, &runtime, a, b, "ab", PartitionId(7));
    let pin = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &request)
        .unwrap();
    let roots = vec![a; 4096];
    let refused = request
        .run(ExecutionWorkCeiling::new(Some(100)), |_| {
            pin.complete_relation_walk(&request, scope(&roots, RelationDirection::Forward))
        })
        .unwrap()
        .0
        .unwrap_err();
    assert_eq!(
        refused.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::WorkExhausted
        ),
        "raw duplicate seeds spend original work and return no partial walk"
    );
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let observed = |event| {
        if let PinnedWalkEventForTest::Entity(id) = event {
            assert_eq!(id, a);
            cancellation.cancel();
        }
    };
    let denial = pin
        .complete_relation_walk(
            &ExecutionRequest::serial(&stopped),
            scope(&[a], RelationDirection::Forward).observe_for_test(&observed),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(denial.reads.entity_records, 1);
    assert_eq!(denial.reads.adjacency_lists, 0);
    let caller_stopped = std::cell::Cell::new(false);
    let observer = |event| {
        if let PinnedWalkEventForTest::Entity(_) = event {
            caller_stopped.set(true);
        }
    };
    let control = || {
        if caller_stopped.get() {
            Err(crate::execution::RelationalExecutionDenialCause::Cancelled)
        } else {
            Ok(())
        }
    };
    let caller_denial = pin
        .complete_relation_walk(
            &request,
            scope(&[a], RelationDirection::Forward)
                .with_control(&control)
                .observe_for_test(&observer),
        )
        .unwrap_err();
    assert_eq!(
        caller_denial.kind,
        PinnedStructuralWalkDenialKind::Read(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(caller_denial.reads.entity_records, 1);
    assert_eq!(caller_denial.reads.adjacency_lists, 0);
    // The same original request succeeds when the separate caller control permits it.
    let walk = pin
        .complete_relation_walk(&request, scope(&roots, RelationDirection::Forward))
        .unwrap();
    let payer = walk._retention.as_ref().expect("original serial payer");
    assert!(payer.bytes() > 0);
    assert_eq!(walk.visits().len(), 2);
}

#[test]
fn optional_and_ancestral_memory_custody_uses_the_original_backing() {
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let setup = crate::test_execution_policy::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&setup_request, &runtime, "a", PartitionId(7));
    let pin = runtime
        .read_truth()
        .pin_read_capability(
            &runtime.visibility_authority().snapshot(),
            &[],
            &setup_request,
        )
        .unwrap();
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let unaccounted = SerialRequest::from_policy(&policy, CancellationSource::new().token(), None);
    let walk = pin
        .complete_relation_walk(
            &ExecutionRequest::serial(&unaccounted),
            scope(&[a], RelationDirection::Forward),
        )
        .unwrap();
    assert!(walk._retention.is_none(), "absent accounting stays absent");
    drop(walk);
    let parent = SerialMemoryBudget::new(Some(20_000));
    let child = SerialMemoryBudget::new(None);
    let accounted = SerialRequest::from_memory(child, CancellationSource::new().token(), None);
    let parent_host =
        SerialRequest::from_memory(parent.clone(), CancellationSource::new().token(), None);
    let walk = ExecutionRequest::serial(&parent_host)
        .run_native(|_| {
            pin.complete_relation_walk(
                &ExecutionRequest::serial(&accounted),
                scope(&[a], RelationDirection::Forward),
            )
        })
        .unwrap()
        .0
        .unwrap();
    let held = walk._retention.as_ref().unwrap().bytes();
    assert!(held > 0);
    assert_eq!(
        parent.charged_memory_bytes(),
        held,
        "scratch was released at handoff"
    );
    assert!(
        parent.reserve(20_001 - held).is_err(),
        "the held result competes on its ancestor"
    );
    drop(walk);
    assert_eq!(parent.charged_memory_bytes(), 0);
    assert_eq!(parent.reserve(20_000).unwrap().bytes(), 20_000);
}

#[test]
fn leased_walk_preserves_original_stop_partial_reads_and_output_payer() {
    use worth_execution::LeaseRequest;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let setup = crate::test_execution_policy::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a = create_entity_in_partition(&setup_request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&setup_request, &runtime, "b", PartitionId(7));
    let edge = create_relation_in_partition(&setup_request, &runtime, a, b, "ab", PartitionId(7));
    let pin = runtime
        .read_truth()
        .pin_read_capability(
            &runtime.visibility_authority().snapshot(),
            &[],
            &setup_request,
        )
        .unwrap();
    for stop in [true, false] {
        let cancellation = CancellationSource::new();
        let lease = test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(20_000), None),
                ),
                deadline: None,
                cancellation: cancellation.token(),
            })
            .unwrap();
        let execution = ExecutionRequest::leased(&lease);
        let observer = |event| match event {
            PinnedWalkEventForTest::Relation(id) => {
                assert_eq!(id, edge);
                if stop {
                    cancellation.cancel();
                }
            }
            PinnedWalkEventForTest::Adjacency(id) => assert!([a, b].contains(&id)),
            PinnedWalkEventForTest::Entity(_) => {}
        };
        let result = pin.complete_relation_walk(
            &execution,
            scope(&[a], RelationDirection::Forward).observe_for_test(&observer),
        );
        if stop {
            let refusal = result.unwrap_err();
            assert_eq!(
                refusal.kind,
                PinnedStructuralWalkDenialKind::Read(
                    crate::execution::RelationalExecutionDenialCause::Cancelled
                )
            );
            assert_eq!(
                refusal.reads,
                PinnedStructuralWalkReads {
                    entity_records: 1,
                    adjacency_lists: 1,
                    relation_records: 1
                }
            );
            assert_eq!(lease.charged_memory_bytes(), 0);
        } else {
            let walk = result.unwrap();
            assert_eq!(walk.visits().len(), 2);
            assert_eq!(
                lease.charged_memory_bytes(),
                walk._retention.as_ref().unwrap().bytes()
            );
            drop(walk);
            assert_eq!(lease.charged_memory_bytes(), 0);
        }
    }
}
