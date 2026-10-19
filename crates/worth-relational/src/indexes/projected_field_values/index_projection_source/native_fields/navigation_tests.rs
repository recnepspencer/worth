//! The native borrowed field walk admits its real AVL navigation before allocation.
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
    CancellationToken, ExecutionRequest, LeaseRequest, MapKernelFailure, MapStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn lease(limit: Option<u64>) -> worth_execution::ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, limit, None),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
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
    let mut navigation = 0;
    // The original storage owner supplies its actual quote without allocating.
    assert!(slots
        .try_set_slots(|_units, bytes| {
            navigation = bytes;
            Err(())
        })
        .is_err());
    assert!(navigation > 0);
    let probe = lease(None);
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
    for funded in [false, true] {
        let limit = opening.checked_add(navigation).unwrap() - u64::from(!funded);
        let lease = lease(Some(limit));
        let request = ExecutionRequest::leased(&lease);
        let visited = AtomicUsize::new(0);
        let preparation = AtomicUsize::new(0);
        let settlement = AtomicUsize::new(0);
        let work = AtomicU64::new(0);
        let result = execute_read_only_packets(
            packet(),
            &lease,
            |(), context| {
                let before = |units, bytes| {
                    work.fetch_add(units, Ordering::Relaxed);
                    context.checkpoint(units)?;
                    context.settle_temporary_scratch(bytes)?;
                    if units == 0 && bytes != 0 {
                        assert_eq!(bytes, navigation);
                        preparation.fetch_add(1, Ordering::Relaxed);
                    } else if units == 0 {
                        assert_eq!(lease.charged_memory_bytes(), opening);
                        settlement.fetch_add(1, Ordering::Relaxed);
                    }
                    Ok::<_, MapKernelFailure<PacketBudgetDenial>>(())
                };
                let check_live = || {
                    assert_eq!(
                        lease.charged_memory_bytes(),
                        opening + navigation,
                        "the original temporary payer stays live during borrowed visits"
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
        );
        if funded {
            result.unwrap();
            let rows = if relations { 1 } else { 2 };
            assert_eq!(visited.load(Ordering::Relaxed), rows);
            assert_eq!(work.load(Ordering::Relaxed), rows as u64);
            assert_eq!(preparation.load(Ordering::Relaxed), 1);
            assert_eq!(settlement.load(Ordering::Relaxed), 1);
        } else {
            assert!(matches!(result, Err(crate::execution::PacketExecutionStop::Execution {
                reason: MapStop::Failure { cause: MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
                    crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
                )), .. }, ..
            })));
            assert_eq!(
                visited.load(Ordering::Relaxed),
                0,
                "refuse before any native field visit"
            );
            assert_eq!(
                work.load(Ordering::Relaxed),
                0,
                "navigation preparation does no candidate work"
            );
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    release_test_commit_snapshot(&runtime, &created);
}

#[test]
fn native_entity_field_navigation_refuses_before_visit_and_retains_original_payer() {
    native_field_navigation_boundary(false);
}
#[test]
fn native_relation_field_navigation_refuses_before_visit_and_retains_original_payer() {
    native_field_navigation_boundary(true);
}
