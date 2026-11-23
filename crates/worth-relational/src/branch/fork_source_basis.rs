use serde::{Deserialize, Serialize};
use worth_foundational::FoundationalBranchReferenceObservation;
use worth_proof::AuthorityWitness;

use crate::history::data::BranchId;

use super::authority::{RelationalForkSourceAuthority, RelationalForkSourceAuthorityMarker};
use super::target::RelationalBranchTarget;
use crate::branch::version::RelationalBranchVersion;

/// Serializable description of an exact live source observation. It carries
/// no owner authority and cannot be used to fork by itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalForkSourceDescriptor {
    runtime_instance_id: u64,
    pub(crate) source_branch: BranchId,
    observation: FoundationalBranchReferenceObservation<RelationalBranchTarget>,
    truth_version: RelationalBranchVersion,
}

impl RelationalForkSourceDescriptor {
    pub(crate) fn new(
        runtime_instance_id: u64,
        observation: FoundationalBranchReferenceObservation<RelationalBranchTarget>,
        source_branch: BranchId,
        truth_version: RelationalBranchVersion,
    ) -> Self {
        Self {
            runtime_instance_id,
            source_branch,
            observation,
            truth_version,
        }
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn observation(&self) -> &FoundationalBranchReferenceObservation<RelationalBranchTarget> {
        &self.observation
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    /// Exact owner-local truth version observed with the branch reference.
    /// This is descriptive evidence only; it cannot admit a transaction or
    /// fork without the owner-issued basis token.
    pub const fn truth_version(&self) -> RelationalBranchVersion {
        self.truth_version
    }
}

/// Proof-backed Phase-4 source token. It intentionally exposes no serde or
/// cloning implementation and is only consumable by the branch owner.
#[derive(Debug)]
pub struct AdmittedRelationalForkSourceBasis {
    pub(crate) descriptor: RelationalForkSourceDescriptor,
    pub(crate) authority: RelationalForkSourceAuthority,
}

impl AdmittedRelationalForkSourceBasis {
    pub(crate) fn new(
        descriptor: RelationalForkSourceDescriptor,
        authority: AuthorityWitness<RelationalForkSourceAuthorityMarker>,
    ) -> Self {
        Self {
            descriptor,
            authority,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RelationalForkSourceDescriptor,
        AuthorityWitness<RelationalForkSourceAuthorityMarker>,
    ) {
        (self.descriptor, self.authority)
    }

    pub fn descriptor(&self) -> &RelationalForkSourceDescriptor {
        &self.descriptor
    }
}
