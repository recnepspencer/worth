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
