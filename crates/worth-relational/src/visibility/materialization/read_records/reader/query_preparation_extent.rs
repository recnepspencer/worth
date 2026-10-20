//! The immutable query shape declares the buffers claimed during packet assembly.
use super::query_execution::QueryReadExecutionStop;
use super::query_packetization::PacketizedQueryWork;
use super::query_preparation_packetization::encoded_value_bound;
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::transactions::data::RecordRef;

pub(super) fn declared_bytes(
    packet: &PlannedQueryPacket,
    region_count: usize,
) -> Result<u64, QueryReadExecutionStop> {
    extent(packet, region_count).ok_or(QueryReadExecutionStop::PreparationAdmission(
        worth_execution::ScanDenial::MemoryOverflow,
    ))
}

fn bytes<T>(count: usize) -> Option<u64> {
    (count as u64).checked_mul(std::mem::size_of::<T>() as u64)
}

fn extent(packet: &PlannedQueryPacket, region_count: usize) -> Option<u64> {
    match &packet.scope {
        QueryScope::ExplicitTargets { targets } => bytes::<RecordRef>(targets.len())?
            .checked_mul(2)?
            .checked_add(bytes::<PacketizedQueryWork>(targets.len())?),
        QueryScope::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        }
        | QueryScope::IncomingNeighborhood {
            seeds,
            relation_kind_scope,
        }
        | QueryScope::ConnectivityTraversal {
            seeds,
            relation_kind_scope,
            ..
        } => {
            let count = seeds.len().div_ceil(TARGET_TRAVERSAL_SEEDS_PER_PACKET);
            let kinds = relation_kind_scope.as_ref().map_or(0, |kinds| kinds.len());
            bytes::<EntityId>(seeds.len())?
                .checked_mul(2)?
                .checked_add(bytes::<KindId>(kinds.checked_mul(count.checked_add(1)?)?)?)?
                .checked_add(bytes::<PacketizedQueryWork>(count)?)
        }
        QueryScope::EntityKindScan {
            partition_scope, ..
        }
        | QueryScope::RelationKindScan {
            partition_scope, ..
        } => scan_extent(partition_scope, region_count, 0, 0),
        QueryScope::EntityFieldEquals {
            field_locator,
            value,
            partition_scope,
        }
        | QueryScope::RelationFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => {
            let clone = (field_locator.owned_allocation_capacity_bytes() as u64)
                .checked_add(encoded_value_bound(value))?;
            scan_extent(partition_scope, region_count, clone, 0)
        }
        QueryScope::EntityFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        }
        | QueryScope::RelationFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => {
            let keys = values.iter().try_fold(
                bytes::<AuthoritativeFieldComparisonKey>(values.len())?,
                |total, value| total.checked_add(encoded_value_bound(value)),
            )?;
            let clone = keys.checked_add(field_locator.owned_allocation_capacity_bytes() as u64)?;
            scan_extent(partition_scope, region_count, clone, keys)
        }
        QueryScope::AspectFilteredEntities {
            aspect_filter,
            partition_scope,
            ..
        }
        | QueryScope::AspectFilteredRelations {
            aspect_filter,
            partition_scope,
            ..
        } => scan_extent(
            partition_scope,
            region_count,
            aspect_filter.owned_allocation_capacity_bytes() as u64,
            0,
        ),
    }
}

fn scan_extent(
    scope: &Option<std::sync::Arc<[PartitionId]>>,
    region_count: usize,
    per_packet: u64,
    shared: u64,
) -> Option<u64> {
    let count = scope.as_ref().map_or(region_count, |scope| scope.len());
    let partition_tables = if scope.is_none() {
        bytes::<PartitionId>(region_count.checked_mul(2)?)?
    } else {
        0
    };
    bytes::<PacketizedQueryWork>(count)?
        .checked_add((count as u64).checked_mul(per_packet)?)?
        .checked_add(shared)?
        .checked_add(partition_tables)
}
