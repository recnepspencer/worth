//! Complete resident mirror publication, prepared before any durable effect.
use crate::runtime::ResidentMirrorTarget;
use crate::storage::overlay::{
    EntityWorkingSetLayout, PartitionCloneMode, PartitionMutationJournal, PartitionState,
};

pub(crate) struct PreparedMirrorPublication {
    pub(super) partitions: Vec<PlannedPartitionPublication>,
}

pub(super) enum PlannedPartitionPublication {
    New {
        target: ResidentMirrorTarget,
        entity_layout: Option<EntityWorkingSetLayout>,
        entity_slots: usize,
    },
    Existing(PlannedExistingPartitionPublication),
}

pub(super) struct PlannedExistingPartitionPublication {
    pub(super) target: ResidentMirrorTarget,
    pub(super) partition_state: PartitionState,
    pub(super) journal: PartitionMutationJournal,
    pub(super) strategy: PartitionPublicationStrategy,
}

pub(super) enum PartitionPublicationStrategy {
    EntityOnly(EntityWorkingSetLayout),
    HybridGraph {
        entity_layout: Option<EntityWorkingSetLayout>,
    },
    WholePartition,
}

impl PreparedMirrorPublication {
    pub(crate) fn empty() -> Self {
        Self {
            partitions: Vec::new(),
        }
    }

    pub(crate) fn prepare_new(
        &mut self,
        target: ResidentMirrorTarget,
        journal: &PartitionMutationJournal,
    ) {
        let entity_only = entity_only(journal);
        self.partitions.push(PlannedPartitionPublication::New {
            target,
            entity_layout: entity_only.then(|| select_entity_layout(journal.entity_slots.len())),
            entity_slots: journal.entity_slots.len(),
        });
    }

    pub(crate) fn prepare_existing(
        &mut self,
        target: ResidentMirrorTarget,
        clone_mode: PartitionCloneMode,
        partition_state: PartitionState,
        journal: PartitionMutationJournal,
    ) {
        let strategy = select_publication_strategy(clone_mode, &journal);
        self.partitions.push(PlannedPartitionPublication::Existing(
            PlannedExistingPartitionPublication {
                target,
                partition_state,
                journal,
                strategy,
            },
        ));
    }
}

fn entity_only(journal: &PartitionMutationJournal) -> bool {
    !journal.entity_slots.is_empty()
        && journal.relation_slots.is_empty()
        && journal.adjacency_slots.is_empty()
        && journal.reverse_adjacency_slots.is_empty()
}

fn select_publication_strategy(
    clone_mode: PartitionCloneMode,
    journal: &PartitionMutationJournal,
) -> PartitionPublicationStrategy {
    if entity_only(journal) {
        return PartitionPublicationStrategy::EntityOnly(select_entity_layout(
            journal.entity_slots.len(),
        ));
    }
    if matches!(
        clone_mode,
        PartitionCloneMode::GraphSparseEntities | PartitionCloneMode::EntityOnly
    ) {
        return PartitionPublicationStrategy::HybridGraph {
            entity_layout: (!journal.entity_slots.is_empty())
                .then(|| select_entity_layout(journal.entity_slots.len())),
        };
    }
    PartitionPublicationStrategy::WholePartition
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
