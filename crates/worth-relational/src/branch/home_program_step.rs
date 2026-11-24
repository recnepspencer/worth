//! A carried Home step on the exact admitted Branch identity.
use sha2::{Digest, Sha256};
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::{
    HomeProgramTransitionCaller, HomeProgramTransitionStepAssociation,
};

#[derive(Debug)]
pub(crate) struct AdmittedRelationalHomeProgramStep {
    pub(crate) step: HomeProgramTransitionStepAssociation,
    pub(crate) admission: worth_proof::AdmittedHomeProgramTransitionStep<
        crate::branch::basis_admission_identity::RelationalBranchBasisAdmissionIdentity,
    >,
    pub(crate) commitment: [u8; 32],
    pub(crate) association: [u8; 32],
    pub(crate) index_budget: crate::indexes::data::maintenance::DerivedIndexMaintenanceBudget,
    pub(crate) _memory: Option<ExecutionMemoryReservation>,
}
impl AdmittedRelationalHomeProgramStep {
    pub(crate) fn admits_association(&self, bytes: &[u8]) -> bool {
        HomeProgramTransitionCaller::recognizes(bytes)
            && self.association == <[u8; 32]>::from(Sha256::digest(bytes))
    }
}
