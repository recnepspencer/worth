use super::query_packet_scope::{
    canonical_kind_scope, canonical_seed_ids, entity_scan_partitions, relation_scan_partitions,
};
use super::*;
use crate::storage::data::AuthoritativeFieldComparisonKey;

#[derive(Debug, Clone)]
pub(super) enum PacketizedQueryWork {
    ExplicitTargets(Vec<crate::transactions::data::RecordRef>),
    EntityKindScan {
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
    },
    RelationKindScan {
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
    },
    EntityFieldEquals {
        partition_id: crate::identity::data::PartitionId,
        field_locator: worth_foundational::facade::AspectFieldLocator,
        value: AuthoritativeFieldComparisonKey,
    },
    EntityFieldAnyOf {
        partition_id: crate::identity::data::PartitionId,
        field_locator: worth_foundational::facade::AspectFieldLocator,
        values: Vec<AuthoritativeFieldComparisonKey>,
    },
    RelationFieldEquals {
        partition_id: crate::identity::data::PartitionId,
        field_locator: worth_foundational::facade::AspectFieldLocator,
        value: AuthoritativeFieldComparisonKey,
    },
    RelationFieldAnyOf {
        partition_id: crate::identity::data::PartitionId,
        field_locator: worth_foundational::facade::AspectFieldLocator,
        values: Vec<AuthoritativeFieldComparisonKey>,
    },
    AspectFilteredEntities {
        partition_id: crate::identity::data::PartitionId,
        kind_id: Option<crate::identity::data::KindId>,
        aspect_filter: ProjectionAspectFilter,
    },
    AspectFilteredRelations {
        partition_id: crate::identity::data::PartitionId,
        kind_id: Option<crate::identity::data::KindId>,
        aspect_filter: ProjectionAspectFilter,
    },
    OutgoingNeighborhood {
        seeds: Vec<crate::identity::data::EntityId>,
        relation_kind_scope: Option<Vec<crate::identity::data::KindId>>,
    },
    IncomingNeighborhood {
        seeds: Vec<crate::identity::data::EntityId>,
        relation_kind_scope: Option<Vec<crate::identity::data::KindId>>,
    },
    ConnectivityTraversal {
        seeds: Vec<crate::identity::data::EntityId>,
        relation_kind_scope: Option<Vec<crate::identity::data::KindId>>,
        max_depth: Option<u32>,
    },
}

pub(super) fn packetized_query_work(
    packet: &PlannedQueryPacket,
    entity_partitions: &[crate::identity::data::PartitionId],
    relation_partitions: &[crate::identity::data::PartitionId],
) -> Option<Vec<PacketizedQueryWork>> {
    match &packet.scope {
        QueryScope::ExplicitTargets { targets } => Some(packetized_explicit_target_work(targets)),
        QueryScope::EntityKindScan {
            kind_id,
            partition_scope,
        } => Some(
            entity_scan_partitions(partition_scope, entity_partitions)
                .into_iter()
                .map(|partition_id| PacketizedQueryWork::EntityKindScan {
                    partition_id,
                    kind_id: *kind_id,
                })
                .collect(),
        ),
        QueryScope::RelationKindScan {
            kind_id,
            partition_scope,
        } => Some(
            relation_scan_partitions(partition_scope, relation_partitions)
                .into_iter()
                .map(|partition_id| PacketizedQueryWork::RelationKindScan {
                    partition_id,
                    kind_id: *kind_id,
                })
                .collect(),
        ),
        QueryScope::EntityFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => Some(
            entity_scan_partitions(partition_scope, entity_partitions)
                .into_iter()
                .map(|partition_id| PacketizedQueryWork::EntityFieldEquals {
                    partition_id,
                    field_locator: field_locator.clone(),
                    value: AuthoritativeFieldComparisonKey::from_aspect_value(value),
                })
                .collect(),
        ),
        QueryScope::EntityFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => {
            let comparison_keys = canonical_query_comparison_keys(values.as_ref());
            Some(
                entity_scan_partitions(partition_scope, entity_partitions)
                    .into_iter()
                    .map(|partition_id| PacketizedQueryWork::EntityFieldAnyOf {
                        partition_id,
                        field_locator: field_locator.clone(),
                        values: comparison_keys.clone(),
                    })
                    .collect(),
            )
        }
        QueryScope::RelationFieldEquals {
            field_locator,
            value,
            partition_scope,
        } => Some(
            relation_scan_partitions(partition_scope, relation_partitions)
                .into_iter()
                .map(|partition_id| PacketizedQueryWork::RelationFieldEquals {
                    partition_id,
                    field_locator: field_locator.clone(),
                    value: AuthoritativeFieldComparisonKey::from_aspect_value(value),
                })
                .collect(),
        ),
        QueryScope::RelationFieldAnyOf {
            field_locator,
            values,
            partition_scope,
        } => {
            let comparison_keys = canonical_query_comparison_keys(values.as_ref());
            Some(
                relation_scan_partitions(partition_scope, relation_partitions)
                    .into_iter()
                    .map(|partition_id| PacketizedQueryWork::RelationFieldAnyOf {
                        partition_id,
                        field_locator: field_locator.clone(),
                        values: comparison_keys.clone(),
                    })
                    .collect(),
            )
        }
        QueryScope::AspectFilteredEntities {
            kind_id,
            aspect_filter,
            partition_scope,
        } => Some(
            entity_scan_partitions(partition_scope, entity_partitions)
                .into_iter()
                .map(|partition_id| PacketizedQueryWork::AspectFilteredEntities {
                    partition_id,
                    kind_id: *kind_id,
                    aspect_filter: aspect_filter.clone(),
                })
                .collect(),
        ),
        QueryScope::AspectFilteredRelations {
            kind_id,
            aspect_filter,
            partition_scope,
        } => Some(
            relation_scan_partitions(partition_scope, relation_partitions)
                .into_iter()
                .map(
                    |partition_id| PacketizedQueryWork::AspectFilteredRelations {
                        partition_id,
                        kind_id: *kind_id,
                        aspect_filter: aspect_filter.clone(),
                    },
                )
                .collect(),
        ),
        QueryScope::OutgoingNeighborhood { .. }
        | QueryScope::IncomingNeighborhood { .. }
        | QueryScope::ConnectivityTraversal { .. } => packetized_traversal_query_work(packet),
    }
}

fn canonical_query_comparison_keys(
    values: &[worth_foundational::facade::AspectValue],
) -> Vec<AuthoritativeFieldComparisonKey> {
    let mut keys = QueryScope::canonical_value_scope(values)
        .iter()
        .map(AuthoritativeFieldComparisonKey::from_aspect_value)
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

pub(super) fn packetized_explicit_target_work(
    targets: &std::sync::Arc<[crate::transactions::data::RecordRef]>,
) -> Vec<PacketizedQueryWork> {
    let mut per_partition = BTreeMap::<
        crate::identity::data::PartitionId,
        Vec<crate::transactions::data::RecordRef>,
    >::new();
    for target in targets.iter().cloned() {
        match &target {
            crate::transactions::data::RecordRef::Entity(entity_id) => {
                per_partition
                    .entry(entity_id.partition_id)
                    .or_default()
                    .push(target);
            }
            crate::transactions::data::RecordRef::Relation(relation_id) => {
                per_partition
                    .entry(relation_id.partition_id)
                    .or_default()
                    .push(target);
            }
        }
    }

    let mut packets = Vec::new();
    for (_, mut partition_targets) in per_partition {
        partition_targets.sort();
        let packet_count = coarse_preparation_packet_count(
            partition_targets.len(),
            TARGET_PREPARATION_ITEMS_PER_PACKET,
        );
        if packet_count <= 1 {
            packets.push(PacketizedQueryWork::ExplicitTargets(partition_targets));
            continue;
        }

        for chunk in partition_targets.chunks(TARGET_PREPARATION_ITEMS_PER_PACKET) {
            packets.push(PacketizedQueryWork::ExplicitTargets(chunk.to_vec()));
        }
    }

    packets
}

pub(super) fn packetized_traversal_query_work(
    packet: &PlannedQueryPacket,
) -> Option<Vec<PacketizedQueryWork>> {
    match &packet.scope {
        QueryScope::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        } => Some(packetized_traversal_seed_work(
            canonical_seed_ids(seeds),
            |seed_chunk| PacketizedQueryWork::OutgoingNeighborhood {
                seeds: seed_chunk,
                relation_kind_scope: relation_kind_scope.as_ref().map(canonical_kind_scope),
            },
        )),
        QueryScope::IncomingNeighborhood {
            seeds,
            relation_kind_scope,
        } => Some(packetized_traversal_seed_work(
            canonical_seed_ids(seeds),
            |seed_chunk| PacketizedQueryWork::IncomingNeighborhood {
                seeds: seed_chunk,
                relation_kind_scope: relation_kind_scope.as_ref().map(canonical_kind_scope),
            },
        )),
        QueryScope::ConnectivityTraversal {
            seeds,
            relation_kind_scope,
            max_depth,
        } => Some(packetized_traversal_seed_work(
            canonical_seed_ids(seeds),
            |seed_chunk| PacketizedQueryWork::ConnectivityTraversal {
                seeds: seed_chunk,
                relation_kind_scope: relation_kind_scope.as_ref().map(canonical_kind_scope),
                max_depth: *max_depth,
            },
        )),
        _ => None,
    }
}

fn packetized_traversal_seed_work(
    canonical_seeds: Vec<crate::identity::data::EntityId>,
    build: impl Fn(Vec<crate::identity::data::EntityId>) -> PacketizedQueryWork,
) -> Vec<PacketizedQueryWork> {
    if canonical_seeds.is_empty() {
        return Vec::new();
    }

    let packet_count =
        coarse_preparation_packet_count(canonical_seeds.len(), TARGET_TRAVERSAL_SEEDS_PER_PACKET);
    if packet_count <= 1 {
        return vec![build(canonical_seeds)];
    }

    canonical_seeds
        .chunks(TARGET_TRAVERSAL_SEEDS_PER_PACKET)
        .map(|seed_chunk| build(seed_chunk.to_vec()))
        .collect()
}

pub(super) fn partition_count_for_targets(packets: &[PacketizedQueryWork]) -> usize {
    let mut partitions = std::collections::BTreeSet::new();
    for packet in packets {
        match packet {
            PacketizedQueryWork::ExplicitTargets(targets) => {
                for target in targets {
                    match target {
                        crate::transactions::data::RecordRef::Entity(entity_id) => {
                            partitions.insert(entity_id.partition_id);
                        }
                        crate::transactions::data::RecordRef::Relation(relation_id) => {
                            partitions.insert(relation_id.partition_id);
                        }
                    }
                }
            }
            PacketizedQueryWork::EntityKindScan { partition_id, .. }
            | PacketizedQueryWork::RelationKindScan { partition_id, .. }
            | PacketizedQueryWork::EntityFieldEquals { partition_id, .. }
            | PacketizedQueryWork::EntityFieldAnyOf { partition_id, .. }
            | PacketizedQueryWork::RelationFieldEquals { partition_id, .. }
            | PacketizedQueryWork::RelationFieldAnyOf { partition_id, .. }
            | PacketizedQueryWork::AspectFilteredEntities { partition_id, .. }
            | PacketizedQueryWork::AspectFilteredRelations { partition_id, .. } => {
                partitions.insert(*partition_id);
            }
            PacketizedQueryWork::OutgoingNeighborhood { seeds, .. }
            | PacketizedQueryWork::IncomingNeighborhood { seeds, .. }
            | PacketizedQueryWork::ConnectivityTraversal { seeds, .. } => {
                partitions.extend(seeds.iter().map(|entity_id| entity_id.partition_id));
            }
        }
    }
    partitions.len()
}

pub(super) fn packetized_query_item_count(packets: &[PacketizedQueryWork]) -> usize {
    packets
        .iter()
        .map(|packet| match packet {
            PacketizedQueryWork::ExplicitTargets(targets) => targets.len(),
            PacketizedQueryWork::EntityKindScan { .. }
            | PacketizedQueryWork::RelationKindScan { .. }
            | PacketizedQueryWork::EntityFieldEquals { .. }
            | PacketizedQueryWork::RelationFieldEquals { .. }
            | PacketizedQueryWork::AspectFilteredEntities { .. }
            | PacketizedQueryWork::AspectFilteredRelations { .. } => 1,
            PacketizedQueryWork::EntityFieldAnyOf { values, .. }
            | PacketizedQueryWork::RelationFieldAnyOf { values, .. } => values.len(),
            PacketizedQueryWork::OutgoingNeighborhood { seeds, .. }
            | PacketizedQueryWork::IncomingNeighborhood { seeds, .. }
            | PacketizedQueryWork::ConnectivityTraversal { seeds, .. } => seeds.len(),
        })
        .sum()
}

pub(super) fn packetized_fragment_scratch_reuse_count(packets: &[PacketizedQueryWork]) -> usize {
    packets
        .iter()
        .filter(|packet| packet_uses_fragment_scratch(packet))
        .count()
        .saturating_sub(1)
}

fn packet_uses_fragment_scratch(packet: &PacketizedQueryWork) -> bool {
    !matches!(packet, PacketizedQueryWork::ExplicitTargets(_))
}

pub(super) fn packetized_query_peak_width(packets: &[PacketizedQueryWork]) -> usize {
    packets
        .iter()
        .map(|packet| match packet {
            PacketizedQueryWork::ExplicitTargets(targets) => targets.len(),
            PacketizedQueryWork::EntityKindScan { .. }
            | PacketizedQueryWork::RelationKindScan { .. }
            | PacketizedQueryWork::EntityFieldEquals { .. }
            | PacketizedQueryWork::RelationFieldEquals { .. }
            | PacketizedQueryWork::AspectFilteredEntities { .. }
            | PacketizedQueryWork::AspectFilteredRelations { .. } => 1,
            PacketizedQueryWork::EntityFieldAnyOf { values, .. }
            | PacketizedQueryWork::RelationFieldAnyOf { values, .. } => values.len(),
            PacketizedQueryWork::OutgoingNeighborhood { seeds, .. }
            | PacketizedQueryWork::IncomingNeighborhood { seeds, .. }
            | PacketizedQueryWork::ConnectivityTraversal { seeds, .. } => seeds.len(),
        })
        .max()
        .unwrap_or(0)
}

pub(super) fn query_scope_units(packets: &[PacketizedQueryWork]) -> usize {
    partition_count_for_targets(packets).max(usize::from(!packets.is_empty()))
}
