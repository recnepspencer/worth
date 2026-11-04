mod consequence_materialization;
mod construction;
mod mutation_tracking;
mod sequential_apply;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::config::data::AdjacencyPolicy;
use crate::identity::data::PartitionId;

use super::{EntityWorkingSetLayout, PartitionCloneMode, PartitionMutationJournal, PartitionState};

#[derive(Debug, Clone)]
pub(crate) struct WorkingState {
    pub(crate) content: crate::content::ContentPublication,
    pub(crate) adjacency_policy: AdjacencyPolicy,
    pub(crate) clone_mode: PartitionCloneMode,
    pub(crate) entity_working_set_layout: EntityWorkingSetLayout,
    pub(in crate::storage) partitions: BTreeMap<PartitionId, PartitionState>,
    pub(crate) mutation_journal: BTreeMap<PartitionId, PartitionMutationJournal>,
}
