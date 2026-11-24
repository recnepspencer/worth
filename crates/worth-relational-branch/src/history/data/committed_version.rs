use serde::{Deserialize, Serialize};

use super::RelationalCommitReceipt;

/// Ordinary-history evidence for the transaction that created a version.
///
/// This is deliberately smaller than a replay envelope: ordinary consumers
/// can recover commit identity and result cardinality without opening replay
/// authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommittedVersionSummary {
    commit: RelationalCommitReceipt,
    changed_record_count: usize,
}

impl CommittedVersionSummary {
    pub const fn commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }
}

/// Operations used by the Relational owners.
pub trait CommittedVersionSummaryOwnerPort: Sized {
    fn new(commit: RelationalCommitReceipt, changed_record_count: usize) -> Self;
}

impl CommittedVersionSummaryOwnerPort for CommittedVersionSummary {
    fn new(commit: RelationalCommitReceipt, changed_record_count: usize) -> Self {
        Self {
            commit,
            changed_record_count,
        }
    }
}
