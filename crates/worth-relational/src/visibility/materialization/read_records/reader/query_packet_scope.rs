pub(super) fn entity_scan_partitions(
    partition_scope: &Option<std::sync::Arc<[crate::identity::data::PartitionId]>>,
    present_partitions: &[crate::identity::data::PartitionId],
) -> Vec<crate::identity::data::PartitionId> {
    if let Some(partitions) = partition_scope {
        return partitions.iter().copied().collect();
    }

    present_partitions.to_vec()
}

pub(super) fn relation_scan_partitions(
    partition_scope: &Option<std::sync::Arc<[crate::identity::data::PartitionId]>>,
    present_partitions: &[crate::identity::data::PartitionId],
) -> Vec<crate::identity::data::PartitionId> {
    if let Some(partitions) = partition_scope {
        return partitions.iter().copied().collect();
    }

    present_partitions.to_vec()
}

pub(super) fn canonical_seed_ids(
    seeds: &std::sync::Arc<[crate::identity::data::EntityId]>,
) -> Vec<crate::identity::data::EntityId> {
    let mut seeds = seeds.iter().copied().collect::<Vec<_>>();
    seeds.sort();
    seeds.dedup();
    seeds
}

pub(super) fn canonical_kind_scope(
    relation_kind_scope: &std::sync::Arc<[crate::identity::data::KindId]>,
) -> Vec<crate::identity::data::KindId> {
    let mut kinds = relation_kind_scope.iter().copied().collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    kinds
}
