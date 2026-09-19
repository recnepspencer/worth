mod access;
mod allocation;
mod cause_validation_work;
mod conditional_versions;
mod construction;
mod contracts;
mod diagnostic_artifacts;
mod evaluation_mutation;
pub(in crate::data::graph) mod evaluation_payload;
mod invalidation_authority;
mod iteration;
mod node_copy_work;
mod node_mutation_work;
pub(in crate::data::graph) use evaluation_mutation::NodeEvaluationMutation;
mod prepared_invalidation_cache;
pub(crate) use prepared_invalidation_cache::PreparedInvalidationCache;
mod snapshots;
mod transitions;

use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeEntry;
use crate::data::reuse::PersistentCorrespondenceKind;

use std::ops::{Deref, DerefMut};

pub(crate) struct MaterializedEntryRef(NodeEntry);

impl Deref for MaterializedEntryRef {
    type Target = NodeEntry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct MaterializedEntryGuard<'a> {
    graph: &'a mut SignalGraph,
    id: NodeId,
    entry: NodeEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct NodeReplayProjection {
    pub lineage_artifact_id: Option<crate::diagnostics::lineage::LineageArtifactId>,
    pub persistent_correspondence_kind: Option<PersistentCorrespondenceKind>,
    pub composition_region_count: Option<u32>,
}

impl Deref for MaterializedEntryGuard<'_> {
    type Target = NodeEntry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl DerefMut for MaterializedEntryGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}

impl Drop for MaterializedEntryGuard<'_> {
    fn drop(&mut self) {
        let entry = std::mem::take(&mut self.entry);
        self.graph.write_back_materialized_entry(self.id, entry);
    }
}

impl SignalGraph {
    pub(crate) fn get_entry(&self, id: NodeId) -> Result<MaterializedEntryRef, SignalError> {
        Ok(MaterializedEntryRef(self.materialize_entry(id)?))
    }

    pub(crate) fn get_entry_mut(
        &mut self,
        id: NodeId,
    ) -> Result<MaterializedEntryGuard<'_>, SignalError> {
        let entry = self.materialize_entry(id)?;
        Ok(MaterializedEntryGuard {
            graph: self,
            id,
            entry,
        })
    }

    fn materialize_entry(&self, id: NodeId) -> Result<NodeEntry, SignalError> {
        crate::data::access_counters::note_materialized_entry_read();
        Ok(NodeEntry::from_storage_parts(
            self.definition_ref(id)?.clone(),
            self.hot_ref(id)?.clone(),
            self.warm_ref(id)?.clone(),
            self.cold_ref(id)?.map(|cold| Box::new(cold.clone())),
        ))
    }

    fn write_back_materialized_entry(&mut self, id: NodeId, entry: NodeEntry) {
        crate::data::access_counters::note_materialized_entry_write();
        let index = id.index() as usize;
        let (definition, hot, warm, cold) = entry.into_storage_parts();
        if self.arena.definitions[index] != definition {
            self.arena.definitions[index] = definition;
        }
        self.arena.hot[index] = Some(hot);
        self.arena.warm[index] = warm;
        self.arena.cold[index] = cold;
    }
}
