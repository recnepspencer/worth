use crate::identity::data::{LineageId, RelationId};
use crate::runtime::RelationalRuntime;

pub(crate) struct LineageAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn lineage_authority(&self) -> LineageAuthority<'_> {
        LineageAuthority { runtime: self }
    }
}

impl<'runtime> LineageAuthority<'runtime> {
    pub(crate) fn install_published_lineage(
        &self,
        events: crate::runtime::ValidatedLineageEventBatch,
        publication_commit_id: crate::history::data::CommitId,
        new_nodes: impl IntoIterator<Item = crate::lineage::data::LineageNode>,
        new_relation_lineages: impl IntoIterator<Item = (RelationId, LineageId)>,
    ) {
        for node in new_nodes {
            self.runtime.lineage.record_node(node);
        }
        for (relation_id, lineage_id) in new_relation_lineages {
            self.runtime
                .lineage
                .record_relation_lineage(relation_id, lineage_id);
        }
        self.runtime
            .lineage
            .install_validated_event_batch(events, publication_commit_id);
    }
}
