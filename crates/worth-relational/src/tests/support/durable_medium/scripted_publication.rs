//! Scripted answers for the next publication, standing in for a home whose
//! journal append failed before or after its effect.

use super::journal::TestDurableJournal;
use super::transition_journal::TestTransitionJournal;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::DurablePublicationFate;

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
    ) -> DurablePublicationFate<String> {
        if self == Self::IndeterminateAfterEffect && journal.is_next(artifact.position()) {
            journal.store(artifact);
        }
        self.publication()
    }

    pub(super) fn answer_transition(
        self,
        journal: &mut TestTransitionJournal,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        if self == Self::IndeterminateAfterEffect && journal.is_next(artifact.sequence()) {
            journal.store(artifact);
        }
        self.publication()
    }

    fn publication(self) -> DurablePublicationFate<String> {
        match self {
            Self::ProvenNoEffect => DurablePublicationFate::ProvenNoEffect {
                detail: "scripted: the journal refused before appending".to_owned(),
            },
            Self::IndeterminateWithoutEffect => DurablePublicationFate::Indeterminate {
                detail: "scripted: the append's fate is unknown (not appended)".to_owned(),
            },
            Self::IndeterminateAfterEffect => DurablePublicationFate::Indeterminate {
                detail: "scripted: the append's fate is unknown (appended)".to_owned(),
            },
        }
    }
}
