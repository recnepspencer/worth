//! Native borrowed field walks use inline navigation and retain original request stops.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::execution::{
    execute_read_only_packets, PacketBudgetDenial, PacketMemory, ReadOnlyPacket,
};
use crate::identity::data::PartitionId;
use crate::tests::support::*;
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, LeaseRequest, MapKernelFailure,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn lease(
    limit: Option<u64>,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, limit, None),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}
fn packet() -> Vec<ReadOnlyPacket<()>> {
    vec![ReadOnlyPacket {
        identity: PartitionIdentity::new(0),
        value: (),
        input_bytes: 0,
        memory: PacketMemory::Tracked,
    }]
}

fn native_field_navigation_boundary(relations: bool) {
    let setup = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity(&execution, &runtime, "left");
    let right = create_entity(&execution, &runtime, "right");
    let created = create_relation_outcome(&execution, &runtime, left, right, "edge");
    let relation = changed_relations(&created)[0];
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let source = IndexProjectionSource::exact(&view).unwrap();
    let root = source.projection.selected_root().unwrap();
    let partition = root.get_partition(PartitionId::main()).unwrap();
    let slots = if relations {
        &partition.relation_arena.live_bitset
    } else {
        &partition.entity_arena.live_bitset
    };
    let mut navigation = None;
    // The original owner still admits navigation, but has no heap scratch.
    assert!(slots
        .try_set_slots(|_units, bytes| {
            navigation = Some(bytes);
            Err(())
        })
        .is_err());
    assert_eq!(navigation, Some(0));
    let probe = lease(None, CancellationToken::new());
    let opening = AtomicU64::new(0);
    execute_read_only_packets(
        packet(),
        &probe,
        |(), _context| {
            opening.store(probe.charged_memory_bytes(), Ordering::Relaxed);
            Ok(())
        },
        |_| 0,
    )
    .unwrap();
    let opening = opening.load(Ordering::Relaxed);
    assert!(opening > 0);
    let field_name = if relations { "label" } else { "name" };
    let field = aspect_field_locator(aspect_key(field_name), field_key(field_name));
    let left_value = string_aspect_value("left");
    let right_value = string_aspect_value("right");
    let edge_value = string_aspect_value("edge");
    let lease = lease(Some(opening), CancellationToken::new());
    let request = ExecutionRequest::leased(&lease);
    let visited = AtomicUsize::new(0);
    let boundary_polls = AtomicUsize::new(0);
    let work = AtomicU64::new(0);
    execute_read_only_packets(
        packet(),
        &lease,
        |(), context| {
            let before = |units, bytes| {
                assert_eq!(bytes, 0);
                work.fetch_add(units, Ordering::Relaxed);
                context.checkpoint(units)?;
                context.settle_temporary_scratch(bytes)?;
                assert_eq!(lease.charged_memory_bytes(), opening);
                if units == 0 {
                    boundary_polls.fetch_add(1, Ordering::Relaxed);
                }
                Ok::<_, MapKernelFailure<PacketBudgetDenial>>(())
            };
            let check_live = || {
                assert_eq!(
                    lease.charged_memory_bytes(),
                    opening,
                    "borrowed visits need no memory beyond the original packet opening"
                );
                visited.fetch_add(1, Ordering::Relaxed);
            };
            let result = if relations {
                source.try_for_each_relation_field(
                    &request,
                    KindId(2),
                    &field,
                    before,
                    |id, value| {
                        check_live();
                        assert_eq!(id, relation);
                        assert_eq!(value, Some(&edge_value));
                        Ok(())
                    },
                )
            } else {
                source.try_for_each_entity_field(
                    &request,
                    KindId(1),
                    &field,
                    before,
                    |id, value| {
                        check_live();
                        assert!(id == left || id == right);
                        assert_eq!(
                            value,
                            Some(if id == left {
                                &left_value
                            } else {
                                &right_value
                            })
                        );
                        Ok(())
                    },
                )
            };
            result.map_err(crate::indexes::projected_field_values::checked_entry_map::read_stop)
        },
        |_| 0,
    )
    .unwrap();
    let rows = if relations { 1 } else { 2 };
    assert_eq!(visited.load(Ordering::Relaxed), rows);
    assert_eq!(work.load(Ordering::Relaxed), rows as u64);
    assert_eq!(boundary_polls.load(Ordering::Relaxed), 2);
    assert_eq!(lease.charged_memory_bytes(), 0);

    let cancellation = CancellationSource::new();
    let stopped_lease = self::lease(Some(0), cancellation.token());
    let stopped_request = ExecutionRequest::leased(&stopped_lease);
    let mut reached_opening = false;
    let before = |units, bytes| {
        assert_eq!((units, bytes), (0, 0));
        reached_opening = true;
        cancellation.cancel();
        Ok::<_, ()>(())
    };
    let stopped = if relations {
        source.try_for_each_relation_field(&stopped_request, KindId(2), &field, before, |_, _| {
            panic!("the original cancelled request must stop before visiting a field")
        })
    } else {
        source.try_for_each_entity_field(&stopped_request, KindId(1), &field, before, |_, _| {
            panic!("the original cancelled request must stop before visiting a field")
        })
    };
    assert!(reached_opening);
    assert!(matches!(
        stopped,
        Err(
            crate::visibility::materialization::read_records::read_stop::ReadStop::Request(
                crate::execution::RelationalExecutionDenialCause::Cancelled
            )
        )
    ));
    assert_eq!(stopped_lease.charged_memory_bytes(), 0);
    release_test_commit_snapshot(&runtime, &created);
}

#[test]
fn native_entity_field_navigation_uses_inline_stack_and_keeps_request_stops() {
    native_field_navigation_boundary(false);
}
#[test]
fn native_relation_field_navigation_uses_inline_stack_and_keeps_request_stops() {
    native_field_navigation_boundary(true);
}
