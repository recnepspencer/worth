//! Issue funded successor observations only for the receiving runtime's publication.
use super::successor::{fund_and_charge, live_bytes};
use super::*;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Admit the Live component of the successor `candidate` will publish.
    /// The candidate's receipt fixes the successor's branch, parents, and
    /// canonical envelope witness, so this charges exactly what observing that
    /// successor would and binds redemption to that exact successor.
    pub(crate) fn admit_successor_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &crate::mvcc::PreparedRelationalCommitCandidate,
    ) -> Result<AdmittedRelationalMemoryComponent, Refusal> {
        crate::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
        if candidate.runtime_instance_id() != self.runtime_instance_id()
            || !candidate.belongs_to_publication_owner(&self.publication_binding())
        {
            return Err(Refusal::Basis(
                crate::branch::basis_denial::RelationalBranchBasisDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            ));
        }
        let (bytes, expected_root_id, expected_commit_id) = candidate
            .with_successor_shape(|descriptor, parents, root_id, commit_id| {
                live_bytes(descriptor, parents).map(|bytes| (bytes, root_id, commit_id))
            })
            .ok_or(Refusal::Basis(
                crate::branch::basis_denial::RelationalBranchBasisDenial::WrongImmutableTarget,
            ))??;
        let funding = fund_and_charge(execution, bytes)?;
        Ok(AdmittedRelationalMemoryComponent {
            owner: self.services.origin_identity(),
            expected_root_id,
            expected_commit_id,
            bytes,
            funding,
        })
    }

    /// Observe the issued successor on its admitted funding and work. A
    /// foreign or substituted successor is refused, never charged again.
    pub(crate) fn observe_admitted_memory_component(
        &self,
        admitted: AdmittedRelationalMemoryComponent,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        if !admitted.owner.same_origin(&self.services.origin_identity()) {
            return Err(Refusal::Basis(
                crate::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        crate::branch::basis_identity_validation::require_local_branch_identity(
            self.runtime_instance_id(),
            &self.history,
            basis.identity(),
        )
        .map_err(Refusal::Basis)?;
        admitted.verify_successor(basis)?;
        self.live_component(basis, admitted.funding)
    }
}
