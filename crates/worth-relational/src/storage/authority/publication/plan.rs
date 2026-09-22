//! Complete partition-publication classification before storage mutation begins.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::PartitionId;
use crate::storage::overlay::{
    EntityWorkingSetLayout, PartitionCloneMode, PartitionMutationJournal, PartitionState,
};

pub(super) struct PartitionPublicationPlan {
    pub(super) partitions: Vec<PlannedPartitionPublication>,
}

pub(super) struct PlannedPartitionPublication {
    pub(super) partition_id: PartitionId,
    pub(super) partition_state: PartitionState,
    pub(super) journal: PartitionMutationJournal,
    pub(super) strategy: PartitionPublicationStrategy,
}

pub(super) enum PartitionPublicationStrategy {
    MissingPartition {
        entity_layout: Option<EntityWorkingSetLayout>,
    },
    ExistingEntityOnly(EntityWorkingSetLayout),
    ExistingHybridGraph {
        entity_layout: Option<EntityWorkingSetLayout>,
    },
    ExistingWholePartition,
}

pub(super) fn plan_partition_publication(
    clone_mode: PartitionCloneMode,
    existing_partition_ids: &BTreeSet<PartitionId>,
    committed_partitions: BTreeMap<PartitionId, (PartitionState, PartitionMutationJournal)>,
) -> PartitionPublicationPlan {
    let partitions = committed_partitions
        .into_iter()
        .map(
            |(partition_id, (partition_state, journal))| PlannedPartitionPublication {
                partition_id,
                strategy: select_publication_strategy(
                    clone_mode,
                    existing_partition_ids.contains(&partition_id),
                    &journal,
                ),
                partition_state,
                journal,
            },
        )
        .collect();
    PartitionPublicationPlan { partitions }
}

fn select_publication_strategy(
    clone_mode: PartitionCloneMode,
    base_exists: bool,
    journal: &PartitionMutationJournal,
) -> PartitionPublicationStrategy {
    let entity_only = !journal.entity_slots.is_empty()
        && journal.relation_slots.is_empty()
        && journal.adjacency_slots.is_empty()
        && journal.reverse_adjacency_slots.is_empty();
    let entity_layout = entity_only.then(|| select_entity_layout(journal.entity_slots.len()));
    if !base_exists {
        return PartitionPublicationStrategy::MissingPartition { entity_layout };
    }
    if let Some(layout) = entity_layout {
        return PartitionPublicationStrategy::ExistingEntityOnly(layout);
    }
    if matches!(
        clone_mode,
        PartitionCloneMode::GraphSparseEntities | PartitionCloneMode::EntityOnly
    ) {
        return PartitionPublicationStrategy::ExistingHybridGraph {
            entity_layout: (!journal.entity_slots.is_empty())
                .then(|| select_entity_layout(journal.entity_slots.len())),
        };
    }
    PartitionPublicationStrategy::ExistingWholePartition
}

fn select_entity_layout(touched_entity_slots: usize) -> EntityWorkingSetLayout {
    let chunk_width = if touched_entity_slots <= 128 {
        128
    } else if touched_entity_slots <= 512 {
        256
    } else {
        512
    };
    EntityWorkingSetLayout::AoSoACandidate { chunk_width }
}
