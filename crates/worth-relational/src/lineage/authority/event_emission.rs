use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::LineageId;
use crate::lineage::authority::{LineageAuthority, LineagePreparationAuthority};
use crate::lineage::data::{
    LineageDecisionKind, LineageDecisionRecord, LineageEventKind, LineageEventRecord,
};

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

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(super) fn prepare_authoritative_lineage_event(
        &self,
        event_id: u64,
        commit: &RelationalCommitReceipt,
        kind: LineageEventKind,
        sources: Vec<LineageId>,
        targets: Vec<LineageId>,
        name_birth: Option<crate::lineage::data::PersistentNameBirthRecord>,
    ) -> LineageEventRecord {
        LineageEventRecord {
            event_id,
            commit: commit.clone(),
            branch_id: commit.branch_id.clone(),
            kind,
            sources,
            targets,
            name_birth,
        }
    }

    pub(super) fn accepted_decision_record(
        &self,
        kind: LineageDecisionKind,
        event: &LineageEventRecord,
    ) -> LineageDecisionRecord {
        LineageDecisionRecord {
            branch_id: event.branch_id().clone(),
            kind,
            event_id: Some(event.event_id()),
            sources: event.sources().to_vec(),
            targets: event.targets().to_vec(),
        }
    }
}
