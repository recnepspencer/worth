//! Query aftermath meaning carried into Relational-owned commit history.

use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};
use worth_relational::facade::transactions::RecordRef;

/// Query meaning of one co-committed aftermath relation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathCausalRole {
    Undo,
    Redo,
}

impl WorthQueryAftermathCausalRole {
    pub(crate) const fn code(self) -> u64 {
        match self {
            Self::Undo => 1,
            Self::Redo => 2,
        }
    }
}

/// Query-owned semantic fact awaiting an ordinary Relational commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryPendingAftermathCausality {
    role: WorthQueryAftermathCausalRole,
    parent: RelationalCommitReceipt,
}

impl WorthQueryPendingAftermathCausality {
    pub(crate) fn undo_of(parent: RelationalCommitReceipt) -> Self {
        Self {
            role: WorthQueryAftermathCausalRole::Undo,
            parent,
        }
    }

    pub(crate) fn redo_of(parent: RelationalCommitReceipt) -> Self {
        Self {
            role: WorthQueryAftermathCausalRole::Redo,
            parent,
        }
    }

    pub(crate) const fn role(&self) -> WorthQueryAftermathCausalRole {
        self.role
    }

    pub(crate) const fn parent(&self) -> &RelationalCommitReceipt {
        &self.parent
    }

    pub(crate) fn key(&self) -> String {
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
}

impl WorthQueryCommittedAftermathCausality {
    pub(crate) fn seal(
        pending: WorthQueryPendingAftermathCausality,
        child: RelationalCommitReceipt,
        fact_record: RecordRef,
    ) -> Option<Self> {
        (child.branch_id == pending.parent.branch_id
            && child.parents.as_slice() == [pending.parent.commit_id])
        .then_some(Self {
            role: pending.role,
            parent: pending.parent,
            child,
            fact_record,
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

    pub const fn fact_record(&self) -> &RecordRef {
        &self.fact_record
    }

    pub const fn parent_commit_id(&self) -> CommitId {
        self.parent.commit_id
    }

    /// Whether this sealed fact is the one `pending` asked to co-commit.
    pub(crate) fn answers(&self, pending: &WorthQueryPendingAftermathCausality) -> bool {
        self.role == pending.role && self.parent == pending.parent
    }
}
