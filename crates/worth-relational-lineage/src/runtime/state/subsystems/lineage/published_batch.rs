//! Publish a validated event batch with its exact lineage nodes and relation links.
use super::{LineageSubsystem, ValidatedLineageEventBatch};
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::RelationId;

impl LineageSubsystem {
    pub fn install_published_lineage(
        &self,
        events: ValidatedLineageEventBatch,
        publication_commit_id: CommitId,
        new_nodes: impl IntoIterator<Item = crate::contracts::lineage::data::graph::LineageNode>,
        new_relation_lineages: impl IntoIterator<Item = (RelationId, LineageId)>,
    ) {
        for node in new_nodes {
            self.record_node(node);
        }
        for (relation_id, lineage_id) in new_relation_lineages {
            self.record_relation_lineage(relation_id, lineage_id);
        }
        self.install_validated_event_batch(events, publication_commit_id);
    }
}
