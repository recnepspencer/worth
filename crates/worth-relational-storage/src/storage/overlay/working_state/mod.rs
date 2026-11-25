#[cfg(test)]
mod consequence_courts;
pub mod consequence_materialization;
pub mod construction;
pub mod mutation_tracking;
pub mod request_mutation;
pub mod retraction;
pub mod sequential_apply;
#[cfg(test)]
mod tests;

use crate::storage::substrate::SharedMap;

use crate::config::data::AdjacencyPolicy;
use crate::identity::data::PartitionId;

use super::{EntityWorkingSetLayout, PartitionCloneMode, PartitionMutationJournal, PartitionState};

#[derive(Debug, Clone)]
pub struct WorkingState {
    pub adjacency_policy: AdjacencyPolicy,
    pub clone_mode: PartitionCloneMode,
    pub entity_working_set_layout: EntityWorkingSetLayout,
    pub(in crate::internal_port::storage) partitions: SharedMap<PartitionId, PartitionState>,
    pub mutation_journal: SharedMap<PartitionId, PartitionMutationJournal>,
}
