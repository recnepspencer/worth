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

use super::basis_descriptor::{
    RelationalBranchBasisDescriptor, RelationalLiveBranchBasisDescriptorAxes,
};
use super::reference_checkpoint_data::canonical_observation;
pub(crate) use super::reference_checkpoint_data::CANONICAL_RUNTIME_INSTANCE_ID;
use crate::branch::reference::observation::RelationalBranchObservationConstructionDenial;

/// Recorded basis descriptors carry no process-local root allocation: root
/// ids are issued by each process and are not durable history.
const CANONICAL_ROOT_IDENTITY: u64 = 0;

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

#[cfg(test)]
#[path = "canonical_runtime_frame_tests.rs"]
mod tests;
