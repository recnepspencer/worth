//! Exact live source binding for a Home-issued planning request, never DATA authority.
use super::*;
use crate::branch::RelationalBranchBasisAdmissionIdentity;
use worth_foundational::facade::DurableWorldOwnerIdentity;
use worth_proof::AdmittedHomeProgramTransitionPlanningRequest;

/// Port-free request binding. The attached Home seal, not this DATA constructor,
/// authenticates the actual owner that supplied these coordinates.
#[derive(Clone, Debug)]
pub struct RelationalHomeProgramPlanningSource {
    actor: DurableWorldOwnerIdentity,
    basis: RelationalBranchBasisAdmissionIdentity,
}
impl RelationalHomeProgramPlanningSource {
    pub fn from_owner_source(
        actor: DurableWorldOwnerIdentity,
        basis: RelationalBranchBasisAdmissionIdentity,
    ) -> Self {
        Self { actor, basis }
    }
    pub const fn actor(&self) -> DurableWorldOwnerIdentity {
        self.actor
    }
}
impl RelationalRuntime {
    /// Admit owner and incarnation first, then the complete original basis.
    /// Returns recorded actor DATA; this does not execute a step or publish.
    pub fn admit_home_program_transition_planning_request(
        &self,
        request: &AdmittedHomeProgramTransitionPlanningRequest<RelationalHomeProgramPlanningSource>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<DurableWorldOwnerIdentity, RelationalHomeProgramContractDenial> {
        use RelationalHomeProgramContractDenial as Denial;
        let medium = self.durability.medium().ok_or(Denial::UnboundHome)?;
        medium
            .home_transition_authority()
            .ok_or(Denial::UnboundHome)?
            .admit_planning_request(request, request.source().actor.home().bytes())
            .map_err(Denial::Admission)?;
        medium
            .admit_home_program_planning()
            .map_err(Denial::TransitionPhase)?;
        if basis.descriptor().runtime_instance_id() != self.runtime_instance_id() {
            return Err(Denial::ForeignRuntime);
        }
        if &request.source().basis != basis.admission_identity() {
            return Err(Denial::SourceBasisMismatch);
        }
        Ok(request.source().actor)
    }
}
