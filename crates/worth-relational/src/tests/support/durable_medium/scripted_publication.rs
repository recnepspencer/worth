//! Scripted answers for the next publication, standing in for a home whose
//! journal append failed before or after its effect.

use super::journal::TestDurableJournal;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::RelationalDurablePublication;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptedPublication {
    /// Nothing becomes durable and the medium proves it.
    ProvenNoEffect,
    /// Nothing becomes durable, but the medium cannot tell.
    IndeterminateWithoutEffect,
    /// The artifact becomes durable, but the medium cannot tell.
    IndeterminateAfterEffect,
}

impl ScriptedPublication {
    pub(super) fn answer(
        self,
        journal: &mut TestDurableJournal,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        match self {
            Self::ProvenNoEffect => RelationalDurablePublication::ProvenNoEffect {
                detail: "scripted: the journal refused before appending".to_owned(),
            },
            Self::IndeterminateWithoutEffect => RelationalDurablePublication::Indeterminate {
                detail: "scripted: the append's fate is unknown (not appended)".to_owned(),
            },
            Self::IndeterminateAfterEffect => {
                if journal.is_next(artifact.position()) {
                    journal.store(artifact);
                }
                RelationalDurablePublication::Indeterminate {
                    detail: "scripted: the append's fate is unknown (appended)".to_owned(),
                }
            }
        }
    }
}
