//! Select authenticated scan regions and admit packet preparation work.
use super::super::query_preparation::QueryPreparationBudget;
use crate::query::data::planned_packets::QueryScope;
use crate::storage::overlay::PartitionAccess;
use crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;

pub fn query_has_explicit_partition_scope(scope: &QueryScope) -> bool {
    query_partition_scope(scope).is_some()
}
pub fn query_partition_scope(scope: &QueryScope) -> Option<&[crate::identity::data::PartitionId]> {
    match scope {
        QueryScope::EntityKindScan {
            partition_scope, ..
        }
        | QueryScope::RelationKindScan {
            partition_scope, ..
        }
        | QueryScope::EntityFieldEquals {
            partition_scope, ..
        }
        | QueryScope::RelationFieldEquals {
            partition_scope, ..
        }
        | QueryScope::EntityFieldAnyOf {
            partition_scope, ..
        }
        | QueryScope::RelationFieldAnyOf {
            partition_scope, ..
        }
        | QueryScope::AspectFilteredEntities {
            partition_scope, ..
        }
        | QueryScope::AspectFilteredRelations {
            partition_scope, ..
        } => partition_scope.as_deref(),
        _ => None,
    }
}

pub fn leased_scan_partitions(
    basis: &ResidentVisibilitySnapshotBasis,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<
    (
        Vec<crate::identity::data::PartitionId>,
        Vec<crate::identity::data::PartitionId>,
    ),
    worth_execution::MapKernelFailure<()>,
> {
    let root = basis.projection_root();
    let region_count = root.region_count();
    budget.claim_items::<crate::identity::data::PartitionId>(region_count.saturating_mul(2))?;
    let mut entities = Vec::with_capacity(region_count);
    let mut relations = Vec::with_capacity(region_count);
    root.try_for_each_partition_id(
        |partition_id| -> Result<(), worth_execution::MapKernelFailure<()>> {
            budget.checkpoint(1)?;
            let Some(partition) = basis.get_partition(partition_id) else {
                return Ok(());
            };
            let entity_present = partition.entity_arena.live_bitset.count_ones() > 0;
            let relation_present = partition.relation_arena.live_bitset.count_ones() > 0;
            if entity_present {
                entities.push(partition_id);
            }
            if relation_present {
                relations.push(partition_id);
            }
            Ok(())
        },
    )?;
    Ok((entities, relations))
}

pub fn query_kind(scope: &QueryScope) -> Option<(bool, crate::identity::data::KindId)> {
    match scope {
        QueryScope::EntityKindScan { kind_id, .. }
        | QueryScope::AspectFilteredEntities {
            kind_id: Some(kind_id),
            ..
        } => Some((true, *kind_id)),
        QueryScope::RelationKindScan { kind_id, .. }
        | QueryScope::AspectFilteredRelations {
            kind_id: Some(kind_id),
            ..
        } => Some((false, *kind_id)),
        _ => None,
    }
}
pub fn kind_regions(
    basis: &ResidentVisibilitySnapshotBasis,
    entity: bool,
    kind: crate::identity::data::KindId,
) -> Box<dyn Iterator<Item = crate::identity::data::PartitionId> + '_> {
    let root = basis.projection_root();
    if entity {
        Box::new(root.entity_regions_of_kind(kind))
    } else {
        Box::new(root.relation_regions_of_kind(kind))
    }
}
pub fn leased_kind_partitions(
    basis: &ResidentVisibilitySnapshotBasis,
    entity: bool,
    kind: crate::identity::data::KindId,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<Vec<crate::identity::data::PartitionId>, worth_execution::MapKernelFailure<()>> {
    let count = kind_regions(basis, entity, kind).count();
    budget.claim_items::<crate::identity::data::PartitionId>(count)?;
    let mut partitions = Vec::with_capacity(count);
    for id in kind_regions(basis, entity, kind) {
        budget.checkpoint(1)?;
        partitions.push(id);
    }
    Ok(partitions)
}
