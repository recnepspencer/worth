//! Publish a validated event batch with its exact lineage nodes and relation links.
use super::LineageSubsystem;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::RelationId;
use crate::contracts::lineage::data::events::LineageEventRecord;

pub struct ValidatedLineageEventBatch {
    pub(super) events: Vec<LineageEventRecord>,
    pub(super) groups: Option<(
        BranchId,
        Vec<crate::lineage::groups::CommittedGroupTransition>,
    )>,
}

impl ValidatedLineageEventBatch {
    pub fn from_reserved(events: Vec<LineageEventRecord>) -> Self {
        Self {
            events,
            groups: None,
        }
    }

    /// Adds `branch`'s decoded canonical group lineage frame.
    pub fn with_group_lineage(
        mut self,
        branch: &BranchId,
        frame: &[u8],
    ) -> Result<Self, crate::durability::naming::PersistentNameDurabilityError> {
        let transitions = crate::durability::naming::decode_group_lineage_frame(frame)?;
        self.groups = Some((branch.clone(), transitions));
        Ok(self)
    }
}

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
