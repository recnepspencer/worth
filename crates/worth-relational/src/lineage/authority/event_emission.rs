#[cfg(test)]
use crate::history::data::RelationalCommitReceipt;
#[cfg(test)]
use crate::identity::data::LineageId;
use crate::lineage::authority::LineageAuthority;
#[cfg(test)]
use crate::lineage::authority::LineagePreparationAuthority;
#[cfg(test)]
use crate::lineage::data::{
    LineageDecisionKind, LineageDecisionRecord, LineageEventKind, LineageEventRecord,
    LineageSubject, PersistentNameBirthRecord, PersistentNameIdentity,
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

#[cfg(test)]
impl<'runtime> LineagePreparationAuthority<'runtime> {
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_authoritative_lineage_event(
        &self,
        event_id: u64,
        commit: &RelationalCommitReceipt,
        kind: LineageEventKind,
        subject: LineageSubject,
        sources: Vec<LineageId>,
        targets: Vec<LineageId>,
        name_birth: Option<PersistentNameBirthRecord>,
        kept_name: Option<PersistentNameIdentity>,
    ) -> LineageEventRecord {
        LineageEventRecord {
            event_id,
            commit: commit.clone(),
            branch_id: commit.branch_id.clone(),
            kind,
            sources,
            targets,
            subject,
            name_birth,
            kept_name,
        }
    }

    #[cfg(test)]
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
