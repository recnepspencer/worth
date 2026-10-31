use std::sync::Arc;
use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion,
    CompositeAttemptAssociation, DurableCompositeCommitIdentity, DurableProductBranchIdentity,
    DurableProductBranchIncarnation, DurableProductReferenceGeneration,
    DurableProductUnpublishedOwnerEffectsIdentity, DurableWorldOwnerIdentity, HomeIdentity,
    HomeIncarnation, WorldDurableJournal,
};
pub const WORLD_ARTIFACT_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.world.artifact");
pub const WORLD_ARTIFACT_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(2);

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
        relational: worth_relational::facade::branch::RecordedRelationalComponentReference,
    },
    OpenAttempt(WorldOpenAttemptRecord),
    Occurrence {
        open: WorldOpenAttemptRecord,
        relational: WorldRelationalEvidence,
        publishing_incarnation: HomeIncarnation,
    },
    Released {
        association: CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
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
            Self::Owner { .. } | Self::Branch { .. } => WorldDurableJournal::Branches,
            Self::OpenAttempt(_) | Self::Released { .. } | Self::Continued { .. } => {
                WorldDurableJournal::Custody
            }
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
    pub(crate) record: WorldRecord,
    bytes: Arc<worth_execution::ExecutionArray<u8>>,
}
impl WorldDurableArtifact {
    /// Whether both values retain the same original owner-framed allocation.
    /// This correlation grants no publication or recovery authority.
    pub fn same_subject(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bytes, &other.bytes)
    }
    pub(crate) fn from_record(
        record: WorldRecord,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldDurableMediumDenial> {
        let bytes = Arc::new(super::codec::encode(&record, execution)?);
        Ok(Self { record, bytes })
    }
    pub(crate) fn readmit(
        bytes: &[u8],
        home: HomeIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldDurableMediumDenial> {
        let record =
            super::codec::decode(bytes, home).map_err(super::WorldDurableMediumDenial::Frame)?;
        Self::from_record(record, execution)
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.elements()
    }
    pub fn journal(&self) -> WorldDurableJournal {
        self.record.journal()
    }
}

impl worth_foundational::facade::DurableArtifactRetention for WorldDurableArtifact {
    const RETENTION_CLASS: worth_foundational::facade::RetentionClass =
        worth_foundational::facade::RetentionClass::Authoritative;
}
