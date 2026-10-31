//! Enrollment correspondence consumes actual native admission and selected custody.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};

type Model<S, M> =
    <<M as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Called after fresh document READ and binding-selected action admission.
    /// Success observes correspondence; it retains no permission or execution grant.
    pub fn assess_document_history_operation<H, M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        admitted: &WorthQueryAdmittedApplicationOperation<S, M::Operation, M::Input, Model<S, M>>,
    ) -> Result<(), Denial>
    where
        H: ApplicationHistoryBinding<S>,
        M: ApplicationMutationBinding<S>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        if !std::ptr::eq(self, document.application())
            || !std::ptr::eq(self, selected.application())
        {
            return Err(Denial::ForeignRuntime);
        }
        if document.binding != self.installed_schema().binding_identity()
            || document.binding != *installed.binding_identity()
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        admitted
            .validate_current_authority()
            .map_err(Denial::OperationAdmission)?;
        if !document.model.matches_operation(admitted) {
            return Err(Denial::ForeignModel);
        }
        if !installed.contains_operation::<M>() {
            return Err(Denial::OperationNotDeclared);
        }
        let product = selected.product().read_lease_ref();
        if WorthQueryProductBranchReadIdentity::from_observation(product.observation())
            != document.expected
        {
            return Err(Denial::HeadDiverged);
        }
        current_head::confirm(self, phase, product, ()).map_err(|(_, denial)| denial)
    }
}
