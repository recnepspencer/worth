use crate::lineage::authority::LineageAuthority;

impl<'runtime> LineageAuthority<'runtime> {
    pub(crate) fn install_published_lineage(
        &self,
        events: crate::runtime::ValidatedLineageEventBatch,
        publication_commit_id: crate::history::data::CommitId,
        new_nodes: impl IntoIterator<Item = crate::lineage::data::LineageNode>,
    ) {
        for node in new_nodes {
            self.runtime.lineage.record_node(node);
        }
        self.runtime
            .lineage
            .install_validated_event_batch(events, publication_commit_id);
    }
}
