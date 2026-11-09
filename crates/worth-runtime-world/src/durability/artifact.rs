use super::WorldRelationalReleaseEvidence;
use std::sync::Arc;
mod creation;
mod owner_evidence;
pub(crate) use creation::{
    WorldBranchCreatedRecord, WorldCreatedBranchOwnerEvidence, WorldCreationIntentRecord,
    WorldRecordedRelationalCreationPlan, WorldRelationalForkEvidence,
};
pub(crate) use owner_evidence::{WorldOccurrenceOwnerEvidence, WorldSignalEvidence};
mod record_input;
pub(crate) use record_input::{
    WorldArtifactInput, WorldCreationIntentInput, WorldHeadInput, WorldOpenAttemptInput,
    WorldRelationalCreationInput,
};
mod record_custody;
pub(crate) use record_custody::PreparedWorldArtifact;
pub(crate) use record_custody::WorldArtifactFrame;
#[cfg(test)]
mod retention_tests;
use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion,
    CompositeAttemptAssociation, DurableCompositeCommitIdentity, DurableProductBranchIdentity,
    DurableProductBranchIncarnation, DurableProductReferenceGeneration,
    DurableProductUnpublishedOwnerEffectsIdentity, DurableWorldOwnerIdentity, HomeIncarnation,
    WorldDurableJournal,
};
pub const WORLD_ARTIFACT_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.world.artifact");
pub const WORLD_ARTIFACT_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(5);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurableWorldHead {
    pub(crate) branch: DurableProductBranchIdentity,
    pub(crate) incarnation: DurableProductBranchIncarnation,
    pub(crate) generation: DurableProductReferenceGeneration,
    pub(crate) commit: DurableCompositeCommitIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldOpenAttemptRecord {
    pub(crate) component_intent: WorldOpenComponentIntent,
    pub(crate) association: CompositeAttemptAssociation,
    pub(crate) commit: DurableCompositeCommitIdentity,
    pub(crate) custody: DurableProductUnpublishedOwnerEffectsIdentity,
    pub(crate) expected: DurableWorldHead,
    pub(crate) program_revision: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldOpenComponentIntent {
    RelationalOnly,
    SignalOnly,
    RelationalAndSignal,
}
impl WorldOpenComponentIntent {
    pub(crate) const fn changes_relational(self) -> bool {
        match self {
            Self::RelationalOnly | Self::RelationalAndSignal => true,
            Self::SignalOnly => false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldRelationalEvidence {
    pub(crate) commit: u64,
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldRecord {
    Owner {
        owner: DurableWorldOwnerIdentity,
    },
    Branch {
        head: DurableWorldHead,
        bootstrap: worth_foundational::facade::DurableWorldBootstrapAttemptIdentity,
        program_revision: [u8; 32],
        components: super::RecordedWorldInitialComponents,
    },
    OpenAttempt(WorldOpenAttemptRecord),
    CreationIntent(WorldCreationIntentRecord),
    BranchCreated(WorldBranchCreatedRecord),
    CreationReleased {
        association: CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    },
    Occurrence {
        open: WorldOpenAttemptRecord,
        owners: WorldOccurrenceOwnerEvidence,
        publishing_incarnation: HomeIncarnation,
    },
    Released {
        association: CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        relational: Option<WorldRelationalReleaseEvidence>,
    },
    Continued {
        association: CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    },
}
impl WorldRecord {
    pub(crate) fn owner(&self) -> DurableWorldOwnerIdentity {
        match self {
            Self::Owner { owner } => *owner,
            Self::Branch { head, .. } => head.branch.owner(),
            Self::CreationIntent(intent) => intent.association.attempt().owner(),
            Self::BranchCreated(created) => created.intent.association.attempt().owner(),
            Self::CreationReleased { association, .. } => association.attempt().owner(),
            Self::OpenAttempt(open) | Self::Occurrence { open, .. } => {
                open.association.attempt().owner()
            }
            Self::Released { association, .. } | Self::Continued { association, .. } => {
                association.attempt().owner()
            }
        }
    }
    pub(crate) const fn journal(&self) -> WorldDurableJournal {
        match self {
            Self::Owner { .. } | Self::Branch { .. } | Self::BranchCreated(_) => {
                WorldDurableJournal::Branches
            }
            Self::OpenAttempt(_)
            | Self::Released { .. }
            | Self::Continued { .. }
            | Self::CreationIntent(_)
            | Self::CreationReleased { .. } => WorldDurableJournal::Custody,
            Self::Occurrence { .. } => WorldDurableJournal::Occurrences,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldDurableFrameDenial {
    Malformed,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    WrongHome,
    IdentityMismatch,
    SizeLimit,
    DigestMismatch,
}
/// An owner-framed publication subject. Bytes are opaque to the Home adapter.
/// Public callers cannot manufacture a writable artifact from descriptive IDs.
#[derive(Clone, Debug)]
pub struct WorldDurableArtifact {
    original: Arc<record_custody::WorldArtifactMeaning>,
}
impl WorldDurableArtifact {
    /// Whether both values retain the same original owner-framed allocation.
    /// This correlation grants no publication or recovery authority.
    pub fn same_subject(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.original, &other.original)
    }
    pub(crate) fn from_input(
        input: WorldArtifactInput<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldDurableMediumDenial> {
        let prepared = PreparedWorldArtifact::prepare(input.name_bytes()?, execution)?;
        let record = input.materialize()?;
        let bytes = super::codec::encode(&record, execution)?.into_immutable_bytes();
        Ok(prepared.complete_owned(record, WorldArtifactFrame::from_encoded(bytes)))
    }
    pub(crate) fn from_readmitted_open(
        source: &super::AdmittedWorldOpenAttempt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldDurableMediumDenial> {
        // The original decoder already admitted and owns these exact names.
        let prepared = PreparedWorldArtifact::prepare(0, execution)?;
        let bytes =
            super::codec::encode(&source.record().record, execution)?.into_immutable_bytes();
        Ok(prepared.complete_readmitted(
            source.record().clone(),
            WorldArtifactFrame::from_encoded(bytes),
        ))
    }
    pub(crate) fn record(&self) -> &WorldRecord {
        self.original.record()
    }
    pub(crate) fn open_attempt(&self) -> Option<&WorldOpenAttemptRecord> {
        match self.record() {
            WorldRecord::OpenAttempt(open) => Some(open),
            WorldRecord::Owner { .. }
            | WorldRecord::Branch { .. }
            | WorldRecord::Occurrence { .. }
            | WorldRecord::Released { .. }
            | WorldRecord::Continued { .. }
            | WorldRecord::CreationIntent(_)
            | WorldRecord::BranchCreated(_)
            | WorldRecord::CreationReleased { .. } => None,
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.original.bytes()
    }
    pub fn journal(&self) -> WorldDurableJournal {
        self.record().journal()
    }
}

impl worth_foundational::facade::DurableArtifactRetention for WorldDurableArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
