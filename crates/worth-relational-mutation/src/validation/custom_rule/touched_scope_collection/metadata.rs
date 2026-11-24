use std::collections::BTreeSet;

use crate::validation::engine::state_view::VisibleRelationMetadata;
use worth_relational_contracts::internal_port::identity::data::primitives::PartitionId;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;

pub(super) fn include_relation_metadata(
    visible_entities: &mut BTreeSet<EntityId>,
    touched_partitions: &mut BTreeSet<PartitionId>,
    metadata: VisibleRelationMetadata,
) {
    visible_entities.insert(metadata.source);
    visible_entities.insert(metadata.target);
    touched_partitions.insert(metadata.relation_id.partition_id);
    touched_partitions.insert(metadata.source.partition_id);
    touched_partitions.insert(metadata.target.partition_id);
}
