//! Provenance of an absent immutable publication edge.
//! A release keeps its independently verified retained source/drop
//! transcript, joined to checkpoint-source head custody when the object's
//! chain starts there.

use worth_store_physical_format::PersistedRecordIdentity;

use super::continuation::VerifiedCheckpointSourceAbsence;

#[derive(Clone, Copy)]
pub(super) enum ReleasedClosureEvidence<'a> {
    FirstPublication,
    RetainedHistory(&'a [PersistedRecordIdentity]),
    /// A release whose per-object chain reaches the checkpoint-source head.
    /// `retained_same_key` is sorted.
    CheckpointHeadAnchored {
        residual: &'a VerifiedCheckpointSourceAbsence,
        retained_same_key: &'a [PersistedRecordIdentity],
    },
}

impl ReleasedClosureEvidence<'_> {
    pub(super) fn has_predecessor(self) -> bool {
        match self {
            Self::FirstPublication => false,
            Self::RetainedHistory(records) => !records.is_empty(),
            Self::CheckpointHeadAnchored { .. } => true,
        }
    }

    pub(super) fn settles_absent_child(self, record: PersistedRecordIdentity) -> bool {
        match self {
            Self::FirstPublication => false,
            Self::RetainedHistory(records) => records.binary_search(&record).is_ok(),
            // Head custody settles only what was already absent at the
            // checkpoint source root. Above it, this key's own retained
            // drops settle. A record the checkpoint still routed and that
            // another key's edge removed is therefore never settled.
            Self::CheckpointHeadAnchored {
                residual,
                retained_same_key,
            } => {
                retained_same_key.binary_search(&record).is_ok()
                    || residual.absent_at_source(record)
            }
        }
    }

    pub(super) fn settles_absent_root(self, record: PersistedRecordIdentity) -> bool {
        match self {
            Self::RetainedHistory(records) => records.binary_search(&record).is_ok(),
            // A nonterminal checkpoint head retains the original graph root,
            // so only this key's drops above the checkpoint settle it.
            Self::CheckpointHeadAnchored {
                retained_same_key, ..
            } => retained_same_key.binary_search(&record).is_ok(),
            Self::FirstPublication => false,
        }
    }
}
