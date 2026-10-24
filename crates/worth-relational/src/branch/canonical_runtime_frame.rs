//! The runtime frame canonical artifacts record branch identity in.
//!
//! Branch identity is runtime-affine: a live cell, observation, target and
//! basis descriptor all carry the owning runtime's process-local instance id.
//! A canonical commit artifact must not: its bytes depend only on durable
//! history, so two processes that execute the same operations produce the same
//! artifacts (Milestone 2 D11). Every runtime identity an envelope records is
//! therefore rebound into one reserved frame that no live runtime is ever
//! issued. Recovery rebinds the recorded frame to the recovering runtime, as it
//! does for any recorded cell, and a live comparison against a recorded
//! identity first moves the live side into this frame.

use worth_foundational::FoundationalBranchTarget;

use super::basis::RelationalLiveBranchBasisDescriptorAxes;
use super::{
    relational_branch_observation, RelationalBranchBasisDescriptor, RelationalBranchCellCheckpoint,
    RelationalBranchObservationConstructionDenial, RelationalBranchReferenceObservation,
};

/// The runtime instance id of the canonical frame. Live runtime ids are issued
/// strictly above it with the same decimal width, so a recorded identity can
/// never be mistaken for a live one and recorded text keeps the live width.
pub(crate) const CANONICAL_RUNTIME_INSTANCE_ID: u64 = 1_000_000_000;

/// Recorded basis descriptors carry no process-local root allocation: root
/// ids are issued by each process and are not durable history.
const CANONICAL_ROOT_IDENTITY: u64 = 0;

/// The transaction id an envelope's merged plan records. Transaction ids are
/// issued per process, one for every transaction begun, reads included, so
/// they are not durable history. Live ids start at one; zero is never issued.
pub(crate) const CANONICAL_TRANSACTION_ID: crate::transactions::data::TransactionId =
    crate::transactions::data::TransactionId(0);

impl RelationalBranchCellCheckpoint {
    /// This cell image with every runtime identity in the canonical frame:
    /// the owner, the observed reference and the fork provenance.
    ///
    /// Cost: O(parent count) to rebuild the two targets.
    pub(crate) fn in_canonical_frame(
        &self,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        let observation = canonical_observation(&self.observation, &self.branch_id.0)?;
        let fork_provenance = match (&self.fork_provenance, &self.fork_source_branch_id) {
            (Some(source), Some(source_branch_id)) => {
                Some(canonical_observation(source, &source_branch_id.0)?)
            }
            (None, None) => None,
            _ => return Err(RelationalBranchObservationConstructionDenial::ForkProvenanceMismatch),
        };
        Ok(Self {
            runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
            branch_id: self.branch_id.clone(),
            observation,
            truth_version: self.truth_version,
            fork_provenance,
            fork_source_branch_id: self.fork_source_branch_id.clone(),
        })
    }
}

impl RelationalBranchBasisDescriptor {
    /// This descriptor as a canonical artifact records it: the runtime and
    /// reference in the canonical frame and no process-local root identity.
    /// Commitments, truth version and posture are durable meaning and stay.
    ///
    /// Cost: O(parent count) to rebuild the reference target.
    pub(crate) fn in_canonical_frame(
        &self,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        Ok(Self::with_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
                branch_id: self.branch_id().clone(),
                reference: canonical_observation(self.reference(), &self.branch_id().0)?,
                truth_version: self.truth_version(),
                root_identity: CANONICAL_ROOT_IDENTITY,
                schema_commitment: self.schema_commitment(),
                visibility_commitment: self.visibility_commitment(),
            },
            self.posture(),
        ))
    }
}

fn canonical_observation(
    observation: &RelationalBranchReferenceObservation,
    branch_name: &str,
) -> Result<RelationalBranchReferenceObservation, RelationalBranchObservationConstructionDenial> {
    let target = match observation.target() {
        FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
        FoundationalBranchTarget::Basis(target) => FoundationalBranchTarget::basis(
            target.rebind_runtime_instance_id(CANONICAL_RUNTIME_INSTANCE_ID),
        ),
    };
    relational_branch_observation(
        CANONICAL_RUNTIME_INSTANCE_ID,
        branch_name,
        target,
        observation.generation(),
    )
}

#[cfg(test)]
#[path = "canonical_runtime_frame_tests.rs"]
mod tests;
