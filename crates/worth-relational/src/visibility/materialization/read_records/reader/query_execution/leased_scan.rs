use std::cell::{Cell, RefCell};

use worth_execution::{MapKernelContext, MapKernelFailure};

use super::super::query_packetization::PacketizedQueryWork;
use super::leased_map::{QueryPacketBudget, QueryReadPacketDenial};
use crate::identity::data::{KindId, PartitionId};
use crate::query::data::{PlannedQueryPacket, QueryFragmentCounters, QueryWorkerFragment};
use crate::runtime::{RelationalRuntime, VisibilityProjectionView};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, entity_query_locus_value,
    relation_query_locus_comparison_key, relation_query_locus_value,
};
use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};

type PacketFailure = MapKernelFailure<QueryReadPacketDenial>;

pub(in crate::visibility::materialization::read_records::reader) fn execute_leased_scan_fragment(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
    packet: &PlannedQueryPacket,
    work: &PacketizedQueryWork,
    ordinal: u64,
    ceiling: u64,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<QueryWorkerFragment, PacketFailure> {
    let projection =
        VisibilityProjectionView::new(runtime, SnapshotStateBasis::Exact(basis.clone()));
    let meter = RefCell::new(QueryPacketBudget::new(context, ceiling)?);
    let candidate_bytes = Cell::new(0);
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let (target_count, touched_partitions) =
        if let Some((partition_id, kind_id, target_count)) = entity_scan_scope(work) {
            projection.try_for_each_entity_record_in(
                partition_id,
                kind_id,
                |bytes| {
                    let mut budget = meter.borrow_mut();
                    budget.checkpoint(1)?;
                    budget.check_temporary_record(bytes)?;
                    candidate_bytes.set(bytes);
                    Ok::<(), PacketFailure>(())
                },
                |record| {
                    let comparison_bytes = entity_comparison_scratch(work, record);
                    meter.borrow().check_temporary_record(
                        candidate_bytes.get().saturating_add(comparison_bytes),
                    )?;
                    if entity_matches(work, record) {
                        meter
                            .borrow_mut()
                            .push_cloned_result(&mut entities, record)?;
                    }
                    Ok(())
                },
            )?;
            (target_count, usize::from(!entities.is_empty()))
        } else if let Some((partition_id, kind_id, target_count)) = relation_scan_scope(work) {
            projection.try_for_each_relation_record_in(
                partition_id,
                kind_id,
                |bytes| {
                    let mut budget = meter.borrow_mut();
                    budget.checkpoint(1)?;
                    budget.check_temporary_record(bytes)?;
                    candidate_bytes.set(bytes);
                    Ok::<(), PacketFailure>(())
                },
                |record| {
                    let comparison_bytes = relation_comparison_scratch(work, record);
                    meter.borrow().check_temporary_record(
                        candidate_bytes.get().saturating_add(comparison_bytes),
                    )?;
                    if relation_matches(work, record) {
                        meter
                            .borrow_mut()
                            .push_cloned_result(&mut relations, record)?;
                    }
                    Ok(())
                },
            )?;
            (target_count, usize::from(!relations.is_empty()))
        } else {
            return Err(MapKernelFailure::Domain(
                QueryReadPacketDenial::FragmentUnavailable,
            ));
        };
    Ok(QueryWorkerFragment {
        plan_key: packet.plan_key,
        fragment_key: crate::query::data::deterministic_query_fragment_key(
            packet.plan_key,
            ordinal,
        ),
        ordering: packet.ordering,
        counters: QueryFragmentCounters {
            target_count,
            authoritative_entity_records_emitted: entities.len(),
            authoritative_relation_records_emitted: relations.len(),
            touched_partitions,
        },
        entities,
        relations,
        traversal_basis: None,
    })
}

fn entity_comparison_scratch(work: &PacketizedQueryWork, record: &EntityReadRecord) -> u64 {
    let field_locator = match work {
        PacketizedQueryWork::EntityFieldEquals { field_locator, .. }
        | PacketizedQueryWork::EntityFieldAnyOf { field_locator, .. } => field_locator,
        _ => return 0,
    };
    entity_query_locus_value(record, field_locator).map_or(0, comparison_key_scratch)
}

fn relation_comparison_scratch(work: &PacketizedQueryWork, record: &RelationReadRecord) -> u64 {
    let field_locator = match work {
        PacketizedQueryWork::RelationFieldEquals { field_locator, .. }
        | PacketizedQueryWork::RelationFieldAnyOf { field_locator, .. } => field_locator,
        _ => return 0,
    };
    relation_query_locus_value(record, field_locator).map_or(0, comparison_key_scratch)
}

fn comparison_key_scratch(value: &worth_foundational::facade::AspectValue) -> u64 {
    // The canonical encoder owns one Vec. Its capacity can grow geometrically;
    // reserve twice the maximal logical width plus the two length prefixes.
    let width = value.semantic_byte_width().saturating_add(8);
    u64::try_from(width.saturating_mul(2)).unwrap_or(u64::MAX)
}

fn entity_scan_scope(work: &PacketizedQueryWork) -> Option<(PartitionId, Option<KindId>, usize)> {
    match work {
        PacketizedQueryWork::EntityKindScan {
            partition_id,
            kind_id,
        } => Some((*partition_id, Some(*kind_id), 0)),
        PacketizedQueryWork::EntityFieldEquals { partition_id, .. } => {
            Some((*partition_id, None, 0))
        }
        PacketizedQueryWork::EntityFieldAnyOf {
            partition_id,
            values,
            ..
        } => Some((*partition_id, None, values.len())),
        PacketizedQueryWork::AspectFilteredEntities {
            partition_id,
            kind_id,
            ..
        } => Some((*partition_id, *kind_id, 0)),
        _ => None,
    }
}

fn relation_scan_scope(work: &PacketizedQueryWork) -> Option<(PartitionId, Option<KindId>, usize)> {
    match work {
        PacketizedQueryWork::RelationKindScan {
            partition_id,
            kind_id,
        } => Some((*partition_id, Some(*kind_id), 0)),
        PacketizedQueryWork::RelationFieldEquals { partition_id, .. } => {
            Some((*partition_id, None, 0))
        }
        PacketizedQueryWork::RelationFieldAnyOf {
            partition_id,
            values,
            ..
        } => Some((*partition_id, None, values.len())),
        PacketizedQueryWork::AspectFilteredRelations {
            partition_id,
            kind_id,
            ..
        } => Some((*partition_id, *kind_id, 0)),
        _ => None,
    }
}

fn entity_matches(work: &PacketizedQueryWork, record: &EntityReadRecord) -> bool {
    match work {
        PacketizedQueryWork::EntityKindScan { kind_id, .. } => record.kind.kind_id == *kind_id,
        PacketizedQueryWork::EntityFieldEquals {
            field_locator,
            value,
            ..
        } => entity_query_locus_comparison_key(record, field_locator).as_ref() == Some(value),
        PacketizedQueryWork::EntityFieldAnyOf {
            field_locator,
            values,
            ..
        } => entity_query_locus_comparison_key(record, field_locator)
            .is_some_and(|value| values.binary_search(&value).is_ok()),
        PacketizedQueryWork::AspectFilteredEntities {
            kind_id,
            aspect_filter,
            ..
        } => {
            kind_id.is_none_or(|kind_id| record.kind.kind_id == kind_id)
                && aspect_filter
                    .matches_authoritative_state(record.authoritative_aspect_state.as_ref())
        }
        _ => false,
    }
}

fn relation_matches(work: &PacketizedQueryWork, record: &RelationReadRecord) -> bool {
    match work {
        PacketizedQueryWork::RelationKindScan { kind_id, .. } => record.kind.kind_id == *kind_id,
        PacketizedQueryWork::RelationFieldEquals {
            field_locator,
            value,
            ..
        } => relation_query_locus_comparison_key(record, field_locator).as_ref() == Some(value),
        PacketizedQueryWork::RelationFieldAnyOf {
            field_locator,
            values,
            ..
        } => relation_query_locus_comparison_key(record, field_locator)
            .is_some_and(|value| values.binary_search(&value).is_ok()),
        PacketizedQueryWork::AspectFilteredRelations {
            kind_id,
            aspect_filter,
            ..
        } => {
            kind_id.is_none_or(|kind_id| record.kind.kind_id == kind_id)
                && aspect_filter
                    .matches_authoritative_state(record.authoritative_aspect_state.as_ref())
        }
        _ => false,
    }
}
