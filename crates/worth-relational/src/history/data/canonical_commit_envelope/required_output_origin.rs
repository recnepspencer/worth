//! Complete producer publication membership, independent of member data patches.
use super::CanonicalCommitEnvelope;
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, KindId, VersionId};
use crate::transactions::data::TransactionCommitError;
use serde::{Deserialize, Serialize};
mod installation;
mod members;
mod payload_coverage;
mod readmission;
mod relations;
mod retention;
pub(crate) use relations::CanonicalRequiredOutputRelation;
use retention::{origin_request, origin_retention_bytes};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CanonicalOutputPublication {
    Birth,
    BirthAfterCodecRetirement { retirement: CommitId },
    Preserve { predecessor: CommitId },
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
