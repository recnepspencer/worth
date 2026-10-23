//! Query aftermath meaning carried into Relational-owned commit history.

use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};
use worth_relational::facade::transactions::RecordRef;

/// Query meaning of one co-committed aftermath relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathCausalRole {
    Undo,
    Redo,
    RecordedReverse,
}

impl WorthQueryAftermathCausalRole {
    pub(crate) const fn code(self) -> u64 {
        match self {
            Self::Undo => 1,
            Self::Redo => 2,
            Self::RecordedReverse => 3,
        }
    }
}

/// Query-owned semantic fact awaiting an ordinary Relational commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryPendingAftermathCausality {
    role: WorthQueryAftermathCausalRole,
    parent: RelationalCommitReceipt,
    target_branch: worth_relational::facade::history::BranchId,
    source: Option<WorthQueryRecordedCorrectionSource>,
    correction: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >,
    >,
}

/// Exact historical source, distinct from the new commit's actual ancestry parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecordedCorrectionSource {
    world: worth_runtime_world::facade::CompositeCommitIdentity,
    relational: RelationalCommitReceipt,
}
impl WorthQueryRecordedCorrectionSource {
    pub fn world(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        &self.world
    }
    pub fn relational(&self) -> &RelationalCommitReceipt {
        &self.relational
    }
}

impl WorthQueryPendingAftermathCausality {
    pub(crate) fn undo_of(parent: RelationalCommitReceipt) -> Self {
        Self {
            target_branch: parent.branch_id.clone(),
            source: None,
            correction: None,
            role: WorthQueryAftermathCausalRole::Undo,
            parent,
        }
    }

    pub(crate) fn redo_of(parent: RelationalCommitReceipt) -> Self {
        Self {
            target_branch: parent.branch_id.clone(),
            source: None,
            correction: None,
            role: WorthQueryAftermathCausalRole::Redo,
            parent,
        }
    }

    pub(crate) fn recorded_reverse(
        correction: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >,
    ) -> Self {
        Self {
            role: WorthQueryAftermathCausalRole::RecordedReverse,
            parent: correction.target_parent().clone(),
            target_branch: correction.target_branch().clone(),
            source: Some(WorthQueryRecordedCorrectionSource {
                world: correction.source_world().clone(),
                relational: correction.source_commit().clone(),
            }),
            correction: Some(correction),
        }
    }

    pub(crate) const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.role
    }

    pub(crate) const fn parent(&self) -> &RelationalCommitReceipt {
        &self.parent
    }
    pub(crate) fn target_branch(&self) -> &worth_relational::facade::history::BranchId {
        &self.target_branch
    }
    pub(crate) fn recorded_source(&self) -> Option<&WorthQueryRecordedCorrectionSource> {
        self.source.as_ref()
    }

    pub(crate) fn key(&self) -> String {
        if let Some(correction) = &self.correction {
            return correction.intent().causal_key();
        }
        format!(
            "{}:{}:{}",
            self.role.code(),
            self.parent.branch_id.0,
            self.parent.commit_id.0
        )
    }
}

/// Query aftermath meaning sealed from one Relational commit and its exact
/// co-committed fact record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedAftermathCausality {
    role: WorthQueryAftermathCausalRole,
    parent: RelationalCommitReceipt,
    child: RelationalCommitReceipt,
    fact_record: RecordRef,
    source: Option<WorthQueryRecordedCorrectionSource>,
}

impl WorthQueryCommittedAftermathCausality {
    pub(crate) fn seal(
        pending: WorthQueryPendingAftermathCausality,
        child: RelationalCommitReceipt,
        fact_record: RecordRef,
    ) -> Option<Self> {
        (child.branch_id == pending.target_branch
            && child.parents.as_slice() == [pending.parent.commit_id])
        .then_some(Self {
            role: pending.role,
            parent: pending.parent,
            child,
            fact_record,
            source: pending.source,
        })
    }

    pub const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.role
    }

    pub const fn parent(&self) -> &RelationalCommitReceipt {
        &self.parent
    }

    pub const fn child(&self) -> &RelationalCommitReceipt {
        &self.child
    }
    pub fn recorded_source(&self) -> Option<&WorthQueryRecordedCorrectionSource> {
        self.source.as_ref()
    }

    pub const fn fact_record(&self) -> &RecordRef {
        &self.fact_record
    }

    pub const fn parent_commit_id(&self) -> CommitId {
        self.parent.commit_id
    }

    /// Whether this sealed fact is the one `pending` asked to co-commit.
    pub(crate) fn answers(&self, pending: &WorthQueryPendingAftermathCausality) -> bool {
        self.role == pending.role
            && self.parent == pending.parent
            && self.source == pending.source
            && self.child.branch_id == pending.target_branch
    }
}
