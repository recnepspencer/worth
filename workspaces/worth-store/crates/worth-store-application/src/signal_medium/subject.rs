//! Original immutable Signal frame retained by its actual physical task.
use worth_foundational::facade::{DurableSignalOwnerIdentity, SignalDurableJournal};
use worth_signal::facade::durability::{
    SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact,
};
#[derive(Clone)]
pub(super) enum SignalPublicationArtifact {
    Definition(SignalDefinitionTransitionArtifact),
    Advance(SignalBranchAdvanceArtifact),
}
impl SignalPublicationArtifact {
    fn digest(&self) -> [u8; 32] {
        match self {
            Self::Definition(a) => a.digest(),
            Self::Advance(a) => a.digest(),
        }
    }
    pub(super) fn owner(&self) -> DurableSignalOwnerIdentity {
        match self {
            Self::Definition(a) => a.identity().owner(),
            Self::Advance(a) => a.identity().owner(),
        }
    }
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Definition(a) => a.as_bytes(),
            Self::Advance(a) => a.as_bytes(),
        }
    }
}
#[derive(Clone)]
pub(super) struct SignalPublicationSubject {
    artifact: SignalPublicationArtifact,
    digest: [u8; 32],
}
impl SignalPublicationSubject {
    pub(super) fn new(artifact: SignalPublicationArtifact) -> Self {
        Self {
            digest: artifact.digest(),
            artifact,
        }
    }
    pub(super) fn as_bytes(&self) -> &[u8] {
        self.artifact.bytes()
    }
    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(super) fn journal(&self) -> SignalDurableJournal {
        match self.artifact {
            SignalPublicationArtifact::Definition(_) => SignalDurableJournal::Definitions,
            SignalPublicationArtifact::Advance(_) => SignalDurableJournal::Advances,
        }
    }
    pub(super) fn same_subject(&self, other: &Self) -> bool {
        self.journal() == other.journal() && self.digest == other.digest
    }
}
