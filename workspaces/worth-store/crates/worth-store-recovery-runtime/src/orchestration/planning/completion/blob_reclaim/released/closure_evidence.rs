//! Provenance of an absent immutable publication edge.
//! Ordinary continuation uses current rooted custody; historical replay
//! keeps its independently verified retained source/drop transcript.

use worth_store_physical_format::PersistedRecordIdentity;

use super::continuation::VerifiedSelectedResidualAbsence;

#[derive(Clone, Copy)]
pub(super) enum ReleasedClosureEvidence<'a> {
    FirstPublication,
    RetainedHistory(&'a [PersistedRecordIdentity]),
    CheckpointResidual(&'a VerifiedSelectedResidualAbsence),
}

impl ReleasedClosureEvidence<'_> {
    pub(super) fn has_predecessor(self) -> bool {
        match self {
            Self::FirstPublication => false,
            Self::RetainedHistory(records) => !records.is_empty(),
            Self::CheckpointResidual(_) => true,
        }
    }

    pub(super) fn settles_absent_child(self, record: PersistedRecordIdentity) -> bool {
        match self {
            Self::FirstPublication => false,
            Self::RetainedHistory(records) => records.binary_search(&record).is_ok(),
            // The caller has matched the capability to the exact selected
            // root/routes before traversal. This is never a live-frame waiver.
            Self::CheckpointResidual(_) => true,
        }
    }

    pub(super) fn settles_absent_root(self, record: PersistedRecordIdentity) -> bool {
        match self {
            Self::RetainedHistory(records) => records.binary_search(&record).is_ok(),
            // A nonterminal current head must retain the original graph root.
            Self::FirstPublication | Self::CheckpointResidual(_) => false,
        }
    }
}
