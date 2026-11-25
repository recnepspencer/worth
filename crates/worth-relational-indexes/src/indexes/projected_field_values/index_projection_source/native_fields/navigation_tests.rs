//! Native borrowed field walks use inline navigation and retain original request stops.
use super::*;
use crate::contracts::execution::packet_memory::PacketMemory;
use crate::contracts::execution::read_only_packets::{PacketBudgetDenial, ReadOnlyPacket, execute_read_only_packets};
use crate::contracts::identity::data::primitives::PartitionId;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, LeaseRequest, MapKernelFailure,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

fn lease(
    authority: &'static worth_execution::ExecutionAuthority,
    limit: Option<u64>,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority
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

#[doc(hidden)]
pub fn verify_native_field_navigation_for_test(
    authority: &'static worth_execution::ExecutionAuthority,
    view: &VisibilityProjectionView,
    left: EntityId,
    right: EntityId,
    relation: RelationId,
    relations: bool,
) {
    let source = IndexProjectionSource::exact(view).unwrap();
    let root = source.projection.selected_partitions().unwrap();
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
    let probe = lease(authority, None, CancellationToken::new());
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
    let field =
        crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
            AspectKey::new(field_name).unwrap(),
            FieldKey::new(field_name).unwrap(),
        );
    let left_value = AspectValue::String(InternedString::Raw("left".into()));
    let right_value = AspectValue::String(InternedString::Raw("right".into()));
    let edge_value = AspectValue::String(InternedString::Raw("edge".into()));
    let lease = lease(authority, Some(opening), CancellationToken::new());
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
    let stopped_lease = self::lease(authority, Some(0), cancellation.token());
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
            worth_relational_branch::internal_port::visibility::materialization::read_records::read_stop::ReadStop::Request(
                crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
            )
        )
    ));
    assert_eq!(stopped_lease.charged_memory_bytes(), 0);
}
