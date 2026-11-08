use crate::basis_lifecycle::BasisOperationLane;

use super::super::projection_lifecycle::{
    WorthQueryLiveProjectionPhase, WorthQueryOperationalProjectionProof,
};
use super::WorthQueryAdmittedProjectionSharing;

pub(crate) struct WorthQueryCheckedSharedOwnerRegistration {
    handle: crate::ordinary::live::WorthQueryManagedLiveHandle,
    receipt: super::super::WorthQueryLiveProjectionReceipt,
    conditional_provenance: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    closure: std::sync::Arc<
        crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    >,
    admission: WorthQueryAdmittedProjectionSharing,
}

pub(super) type RegistrationParts = (
    crate::ordinary::live::WorthQueryManagedLiveHandle,
    super::super::WorthQueryLiveProjectionReceipt,
    Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    std::sync::Arc<crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure>,
    WorthQueryAdmittedProjectionSharing,
);

impl WorthQueryCheckedSharedOwnerRegistration {
    pub(super) fn admit<D, O, F, L: BasisOperationLane>(
        source: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
        proof: &WorthQueryOperationalProjectionProof<WorthQueryLiveProjectionPhase, L>,
        handle: crate::ordinary::live::WorthQueryManagedLiveHandle,
        receipt: super::super::WorthQueryLiveProjectionReceipt,
        conditional_provenance: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
        admission: WorthQueryAdmittedProjectionSharing,
    ) -> Result<Self, RegistrationParts> {
        use crate::domain_installation::operation_authority_chain::operation_phase_basis;
        let affinity = operation_phase_basis(source.bound_operation().authority_proof());
        let exact = receipt.operational_identity() == proof.payload().identity()
            && receipt.resource_name() == handle.name()
            && receipt.settled_identity() == source.identity()
            && admission.readmits_lease(
                source.identity(),
                affinity,
                source.semantic_aspect_dependency_closure(),
            );
        if !exact {
            return Err((
                handle,
                receipt,
                conditional_provenance,
                source.dependency_closure_arc(),
                admission,
            ));
        }
        Ok(Self {
            handle,
            receipt,
            conditional_provenance,
            closure: source.dependency_closure_arc(),
            admission,
        })
    }

    pub(crate) fn handle(&self) -> &crate::ordinary::live::WorthQueryManagedLiveHandle {
        &self.handle
    }

    pub(crate) fn into_parts(self) -> RegistrationParts {
        (
            self.handle,
            self.receipt,
            self.conditional_provenance,
            self.closure,
            self.admission,
        )
    }
}
#[cfg(test)]
#[path = "registration_bundle_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
