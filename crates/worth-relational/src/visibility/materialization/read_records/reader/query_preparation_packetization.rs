use std::sync::Arc;

use worth_execution::MapKernelFailure;
use worth_foundational::facade::AspectValue;

use super::query_packetization::PacketizedQueryWork;
use super::query_preparation::QueryPreparationBudget;
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::transactions::data::RecordRef;

type PreparationResult = Result<Vec<PacketizedQueryWork>, MapKernelFailure<()>>;

pub(super) fn prepare_scan_packets(
    packet: &PlannedQueryPacket,
    entities: &[PartitionId],
    relations: &[PartitionId],
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> PreparationResult {
    match &packet.scope {
        QueryScope::EntityKindScan {
            kind_id,
            partition_scope,
        } => map_partitions(partition_scope, entities, budget, |partition_id, _| {
            Ok(PacketizedQueryWork::EntityKindScan {
                partition_id,
                kind_id: *kind_id,
            })
        }),
        QueryScope::RelationKindScan {
            kind_id,
            partition_scope,
        } => map_partitions(partition_scope, relations, budget, |partition_id, _| {
            Ok(PacketizedQueryWork::RelationKindScan {
                partition_id,
                kind_id: *kind_id,
            })
        }),
        QueryScope::EntityFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => map_partitions(partition_scope, entities, budget, |partition_id, budget| {
            claim_field_clone(field_locator, value, budget)?;
            Ok(PacketizedQueryWork::EntityFieldEquals {
                partition_id,
                field_locator: field_locator.clone(),
                value: AuthoritativeFieldComparisonKey::from_aspect_value(value),
            })
        }),
        QueryScope::RelationFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => map_partitions(
            partition_scope,
            relations,
            budget,
            |partition_id, budget| {
                claim_field_clone(field_locator, value, budget)?;
                Ok(PacketizedQueryWork::RelationFieldEquals {
                    partition_id,
                    field_locator: field_locator.clone(),
                    value: AuthoritativeFieldComparisonKey::from_aspect_value(value),
                })
            },
        ),
        QueryScope::EntityFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => {
            let keys = comparison_keys(values, budget)?;
            map_partitions(partition_scope, entities, budget, |partition_id, budget| {
                claim_key_clone(field_locator, &keys, budget)?;
                Ok(PacketizedQueryWork::EntityFieldAnyOf {
                    partition_id,
                    field_locator: field_locator.clone(),
                    values: keys.clone(),
                })
            })
        }
        QueryScope::RelationFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => {
            let keys = comparison_keys(values, budget)?;
            map_partitions(
                partition_scope,
                relations,
                budget,
                |partition_id, budget| {
                    claim_key_clone(field_locator, &keys, budget)?;
                    Ok(PacketizedQueryWork::RelationFieldAnyOf {
                        partition_id,
                        field_locator: field_locator.clone(),
                        values: keys.clone(),
                    })
                },
            )
        }
        QueryScope::AspectFilteredEntities {
            kind_id,
            aspect_filter,
            partition_scope,
        } => map_partitions(partition_scope, entities, budget, |partition_id, budget| {
            budget.claim(aspect_filter.owned_allocation_capacity_bytes() as u64)?;
            Ok(PacketizedQueryWork::AspectFilteredEntities {
                partition_id,
                kind_id: *kind_id,
                aspect_filter: aspect_filter.clone(),
            })
        }),
        QueryScope::AspectFilteredRelations {
            kind_id,
            aspect_filter,
            partition_scope,
        } => map_partitions(
            partition_scope,
            relations,
            budget,
            |partition_id, budget| {
                budget.claim(aspect_filter.owned_allocation_capacity_bytes() as u64)?;
                Ok(PacketizedQueryWork::AspectFilteredRelations {
                    partition_id,
                    kind_id: *kind_id,
                    aspect_filter: aspect_filter.clone(),
                })
            },
        ),
        _ => Ok(Vec::new()),
    }
}

fn map_partitions(
    scope: &Option<Arc<[PartitionId]>>,
    present: &[PartitionId],
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
    mut build: impl FnMut(
        PartitionId,
        &mut QueryPreparationBudget<'_, '_, '_>,
    ) -> Result<PacketizedQueryWork, MapKernelFailure<()>>,
) -> PreparationResult {
    let partitions = scope.as_deref().unwrap_or(present);
    budget.claim_items::<PacketizedQueryWork>(partitions.len())?;
    let mut packets = Vec::with_capacity(partitions.len());
    for &partition_id in partitions {
        budget.checkpoint(1)?;
        packets.push(build(partition_id, budget)?);
    }
    Ok(packets)
}

fn encoded_value_bound(value: &AspectValue) -> u64 {
    // The canonical wire form adds fixed tags and lengths around at most two
    // owned strings. Vec growth can exceed initialized length, hence 4x.
    (value.owned_allocation_capacity_bytes() as u64)
        .saturating_mul(4)
        .saturating_add(64)
}

fn claim_field_clone(
    locator: &worth_foundational::facade::AspectFieldLocator,
    value: &AspectValue,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<(), MapKernelFailure<()>> {
    budget.claim(locator.owned_allocation_capacity_bytes() as u64)?;
    budget.claim(encoded_value_bound(value))
}

fn comparison_keys(
    values: &[AspectValue],
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<Vec<AuthoritativeFieldComparisonKey>, MapKernelFailure<()>> {
    budget.claim_items::<AuthoritativeFieldComparisonKey>(values.len())?;
    let mut keys = Vec::with_capacity(values.len());
    for value in values {
        budget.checkpoint(1)?;
        budget.claim(encoded_value_bound(value))?;
        keys.push(AuthoritativeFieldComparisonKey::from_aspect_value(value));
    }
    budget.checkpoint(sort_work(keys.len()))?;
    keys.sort();
    keys.dedup();
    budget.checkpoint(0)?;
    Ok(keys)
}

fn claim_key_clone(
    locator: &worth_foundational::facade::AspectFieldLocator,
    keys: &[AuthoritativeFieldComparisonKey],
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> Result<(), MapKernelFailure<()>> {
    budget.checkpoint(u64::try_from(keys.len()).unwrap_or(u64::MAX))?;
    budget.claim(locator.owned_allocation_capacity_bytes() as u64)?;
    budget.claim_items::<AuthoritativeFieldComparisonKey>(keys.len())?;
    for key in keys {
        budget.claim(key.owned_allocation_capacity_bytes())?;
    }
    Ok(())
}

pub(super) fn prepare_explicit_packets(
    targets: &[RecordRef],
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> PreparationResult {
    budget.claim_items::<RecordRef>(targets.len())?;
    let mut sorted = Vec::with_capacity(targets.len());
    for target in targets {
        budget.checkpoint(1)?;
        sorted.push(target.clone());
    }
    budget.checkpoint(sort_work(sorted.len()))?;
    sorted.sort_by_key(|target| (target_partition(target), target.clone()));
    budget.checkpoint(0)?;
    budget.claim_items::<PacketizedQueryWork>(targets.len())?;
    budget.claim_items::<RecordRef>(targets.len())?;
    let mut packets = Vec::with_capacity(targets.len());
    let mut start = 0;
    while start < sorted.len() {
        budget.checkpoint(1)?;
        let partition = target_partition(&sorted[start]);
        let mut end = start;
        while end < sorted.len() && target_partition(&sorted[end]) == partition {
            budget.checkpoint(1)?;
            end += 1;
        }
        for chunk in sorted[start..end].chunks(TARGET_PREPARATION_ITEMS_PER_PACKET) {
            budget.checkpoint(u64::try_from(chunk.len()).unwrap_or(u64::MAX))?;
            packets.push(PacketizedQueryWork::ExplicitTargets(chunk.to_vec()));
        }
        start = end;
    }
    Ok(packets)
}

fn target_partition(target: &RecordRef) -> PartitionId {
    match target {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    }
}

pub(super) fn prepare_traversal_packets(
    packet: &PlannedQueryPacket,
    budget: &mut QueryPreparationBudget<'_, '_, '_>,
) -> PreparationResult {
    let (seeds, kinds) = match &packet.scope {
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
        } => (seeds.as_ref(), relation_kind_scope.as_ref()),
        _ => return Ok(Vec::new()),
    };
    budget.claim_items::<EntityId>(seeds.len())?;
    let mut canonical_seeds = Vec::with_capacity(seeds.len());
    for seed in seeds {
        budget.checkpoint(1)?;
        canonical_seeds.push(*seed);
    }
    budget.checkpoint(sort_work(seeds.len()))?;
    canonical_seeds.sort();
    canonical_seeds.dedup();
    budget.checkpoint(0)?;
    let kinds = if let Some(kinds) = kinds {
        budget.claim_items::<KindId>(kinds.len())?;
        let mut canonical_kinds = Vec::with_capacity(kinds.len());
        for kind in kinds.iter() {
            budget.checkpoint(1)?;
            canonical_kinds.push(*kind);
        }
        budget.checkpoint(sort_work(canonical_kinds.len()))?;
        canonical_kinds.sort();
        canonical_kinds.dedup();
        budget.checkpoint(0)?;
        Some(canonical_kinds)
    } else {
        None
    };
    let count = canonical_seeds
        .len()
        .div_ceil(TARGET_TRAVERSAL_SEEDS_PER_PACKET);
    budget.claim_items::<PacketizedQueryWork>(count)?;
    budget.claim_items::<EntityId>(canonical_seeds.len())?;
    budget.claim_items::<KindId>(count.saturating_mul(kinds.as_ref().map_or(0, Vec::len)))?;
    let mut packets = Vec::with_capacity(count);
    for chunk in canonical_seeds.chunks(TARGET_TRAVERSAL_SEEDS_PER_PACKET) {
        budget.checkpoint(
            u64::try_from(chunk.len() + kinds.as_ref().map_or(0, Vec::len)).unwrap_or(u64::MAX),
        )?;
        let seeds = chunk.to_vec();
        let relation_kind_scope = kinds.clone();
        packets.push(match &packet.scope {
            QueryScope::OutgoingNeighborhood { .. } => PacketizedQueryWork::OutgoingNeighborhood {
                seeds,
                relation_kind_scope,
            },
            QueryScope::IncomingNeighborhood { .. } => PacketizedQueryWork::IncomingNeighborhood {
                seeds,
                relation_kind_scope,
            },
            QueryScope::ConnectivityTraversal { max_depth, .. } => {
                PacketizedQueryWork::ConnectivityTraversal {
                    seeds,
                    relation_kind_scope,
                    max_depth: *max_depth,
                }
            }
            _ => unreachable!("traversal scope checked above"),
        });
    }
    Ok(packets)
}

fn sort_work(count: usize) -> u64 {
    let count = u64::try_from(count).unwrap_or(u64::MAX);
    count.saturating_mul(u64::from(64 - count.max(1).leading_zeros()))
}
