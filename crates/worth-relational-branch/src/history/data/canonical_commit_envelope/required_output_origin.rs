//! Complete producer publication membership, independent of member data patches.
use super::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};
use crate::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, KindId, VersionId};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use serde::{Deserialize, Serialize};
pub mod authored_cascade;
pub mod coupled_authored;
pub mod coupled_authored_header;
pub use coupled_authored::{CanonicalCoupledAuthoredPublication, CanonicalCoupledAuthoredSources};
pub mod authored_restoration;
pub mod historical_selection;
pub use historical_selection::CanonicalHistoricalOutputSelection;
pub mod inputs;
pub mod installation;
pub use authored_cascade::CanonicalAuthoredRetirement;
pub use authored_restoration::CanonicalAuthoredRestoration;
pub use installation::CanonicalProducerRetirement;
pub mod created_relation_members;
#[cfg(test)]
pub(in crate::internal_port::history::data) mod fixture_birth;
pub mod group_selection;
pub mod header_corruption;
pub mod header_work;
pub mod origin_history;
pub use group_selection::{OwnedValidatedOutputOrigins, ValidatedOutputOrigins};
pub use origin_history::OutputOriginHistory;
mod payload_coverage;
pub mod producer_retirement_header;
pub mod readmission;
pub mod relations;
pub mod restoration_candidate;
pub use restoration_candidate::validate as validate_authored_restoration_candidate;
pub mod restoration_header;
pub mod retention;
pub use relations::CanonicalRequiredOutputRelation;
use retention::origin_request;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalOutputPublication {
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
    pub fn predecessor(&self) -> Option<CommitId> {
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
pub struct CanonicalOutputMember {
    pub role: String,
    pub entity_name: String,
    pub id: EntityId,
    pub kind: KindId,
    pub posture: crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture,
    pub birth: CommitId,
    pub birth_producer: String,
    pub birth_dependency: worth_foundational::CanonicalDigestId,
    pub before_write: Option<VersionId>,
    pub after_write: VersionId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalRequiredOutputOrigin {
    pub inputs: Vec<crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning>,
    #[serde(skip)]
    pub _input_memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub commit: crate::history::data::RelationalCommitReceipt,
    pub basis: RelationalBranchBasisDescriptor,
    pub producer: String,
    pub source_scope_owned: bool,
    pub dependency_identity: worth_foundational::CanonicalDigestId,
    pub outputs: Vec<EntityId>,
    pub relations: Vec<CanonicalRequiredOutputRelation>,
    pub meaning: crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    pub publication: CanonicalOutputPublication,
    pub birth: CommitId,
    pub birth_producer: String,
    pub birth_dependency: worth_foundational::CanonicalDigestId,
    pub members: Vec<CanonicalOutputMember>,
    #[serde(skip)]
    pub issuer: Option<worth_proof::InstalledApplicationPublicationAuthority>,
    #[serde(skip)]
    pub _funding: Option<worth_execution::ExecutionMemoryReservation>,
    #[serde(skip)]
    pub _cascade_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PartialEq for CanonicalRequiredOutputOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.inputs == other.inputs
            && self.commit == other.commit
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
    pub fn validate_inputs(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), crate::history::data::CodecRetirementClosureStop> {
        inputs::validate(self, context)
    }

    pub fn retain_cascade_memory(
        &mut self,
        memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
    ) {
        self._cascade_memory = memory;
    }

    /// Only the unique, unpublished decoded owner receives this admission;
    /// no authored meaning or installation authority is changed.
    pub fn retain_decoded_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        self._funding = memory;
    }
    /// Native complete publication identity remains meaningful after current-fact decoding.
    pub fn same_publication(&self, other: &Self) -> bool {
        self.commit == other.commit && self.meaning == other.meaning
    }
}
#[cfg(test)]
#[path = "required_output_origin_inline_type_budget.rs"]
pub mod inline_type_budget;
