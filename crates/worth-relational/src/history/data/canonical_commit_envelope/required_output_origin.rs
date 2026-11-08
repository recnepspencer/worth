//! Complete producer publication membership, independent of member data patches.
use super::CanonicalCommitEnvelope;
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, KindId, VersionId};
use crate::transactions::data::TransactionCommitError;
use serde::{Deserialize, Serialize};
mod authored_cascade;
mod coupled_authored;
mod coupled_authored_header;
mod coupled_source;
pub(crate) use coupled_authored::{
    CanonicalCoupledAuthoredPublication, CanonicalCoupledAuthoredSources,
};
mod authored_restoration;
mod historical_selection;
pub(crate) use historical_selection::CanonicalHistoricalOutputSelection;
mod installation;
pub(crate) use authored_cascade::CanonicalAuthoredRetirement;
pub(crate) use authored_restoration::CanonicalAuthoredRestoration;
pub(crate) use installation::CanonicalProducerRetirement;
mod group_selection;
mod header_corruption;
mod header_work;
mod origin_history;
pub(crate) use group_selection::{OwnedValidatedOutputOrigins, ValidatedOutputOrigins};
pub(crate) use origin_history::OutputOriginHistory;
mod members;
mod payload_coverage;
mod producer_retirement_header;
mod readmission;
mod relations;
mod restoration_candidate;
pub(crate) use restoration_candidate::validate as validate_authored_restoration_candidate;
mod restoration_header;
mod retention;
pub(crate) use relations::CanonicalRequiredOutputRelation;
use retention::{origin_request, origin_retention_bytes};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CanonicalOutputPublication {
    Birth,
    BirthAfterCodecRetirement {
        retirement: CommitId,
    },
    Preserve {
        predecessor: CommitId,
    },
    /// Ordinary authored retirement succeeds the selected publication without
    /// inventing a running producer. Attribution remains predecessor provenance.
    AuthoredRestoration {
        retirement: CommitId,
        predecessor: CommitId,
    },
    ProducerRetirement {
        predecessor: CommitId,
    },
    AuthoredRetirement {
        predecessor: CommitId,
    },
    /// Select the producer's committed version; provenance is this new commit.
    HistoricalSelection {
        selected: CommitId,
        generation: VersionId,
        predecessor: Option<CommitId>,
    },
}

impl CanonicalOutputPublication {
    pub(crate) fn predecessor(&self) -> Option<CommitId> {
        match self {
            Self::Birth | Self::BirthAfterCodecRetirement { .. } => None,
            Self::HistoricalSelection { predecessor, .. } => *predecessor,
            Self::Preserve { predecessor }
            | Self::AuthoredRetirement { predecessor }
            | Self::ProducerRetirement { predecessor }
            | Self::AuthoredRestoration { predecessor, .. } => Some(*predecessor),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalOutputMember {
    pub(crate) role: String,
    pub(crate) entity_name: String,
    pub(crate) id: EntityId,
    pub(crate) kind: KindId,
    pub(crate) posture: crate::mvcc::RequiredOutputRolePosture,
    pub(crate) birth: CommitId,
    pub(crate) birth_producer: String,
    pub(crate) birth_dependency: [u8; 32],
    pub(crate) before_write: Option<VersionId>,
    pub(crate) after_write: VersionId,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalRequiredOutputOrigin {
    pub(crate) commit: crate::history::data::RelationalCommitReceipt,
    pub(crate) basis: RelationalBranchBasisDescriptor,
    pub(crate) producer: String,
    pub(crate) source_scope_owned: bool,
    pub(crate) dependency_identity: [u8; 32],
    pub(crate) outputs: Vec<EntityId>,
    pub(crate) relations: Vec<CanonicalRequiredOutputRelation>,
    pub(crate) meaning: crate::mvcc::RequiredOutputPublicationMeaning,
    pub(crate) publication: CanonicalOutputPublication,
    pub(crate) birth: CommitId,
    pub(crate) birth_producer: String,
    pub(crate) birth_dependency: [u8; 32],
    pub(crate) members: Vec<CanonicalOutputMember>,
    #[serde(skip)]
    pub(crate) issuer: Option<worth_proof::InstalledApplicationPublicationAuthority>,
    #[serde(skip)]
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl PartialEq for CanonicalRequiredOutputOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.commit == other.commit
            && self.basis == other.basis
            && self.source_scope_owned == other.source_scope_owned
            && self.producer == other.producer
            && self.dependency_identity == other.dependency_identity
            && self.outputs == other.outputs
            && self.relations == other.relations
            && self.meaning == other.meaning
            && self.publication == other.publication
            && self.birth == other.birth
            && self.birth_producer == other.birth_producer
            && self.birth_dependency == other.birth_dependency
            && self.members == other.members
    }
}
impl Eq for CanonicalRequiredOutputOrigin {}

impl CanonicalRequiredOutputOrigin {
    /// Native's complete publication coordinate survives current-fact decoding.
    pub(crate) fn same_publication(&self, other: &Self) -> bool {
        self.commit == other.commit && self.meaning == other.meaning
    }
}

mod source_scope_ownership;
