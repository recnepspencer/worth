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
        if installed.definition().parts().ownership.is_empty()
            && !document.model.matches_operation(admitted)
        {
            return Err(Denial::ForeignModel);
        }
        if !installed.contains_operation::<M>() {
            return Err(Denial::OperationNotDeclared);
        }
        let admitted_product = admitted
            .graph_work()
            .mutation_product()
            .ok_or(Denial::ForeignRecording)?
            .publication_binding();
        if admitted_product.observation() != selected.product().observation() {
            return Err(Denial::HeadDiverged);
        }
        self.require_history_operation_ownership(phase, document, installed, selected, admitted)?;
        let product = selected.product().read_lease_ref();
        let expected = WorthQueryProductBranchReadIdentity::from_observation(product.observation());
        if expected != document.selection.lock().expected {
            return Err(Denial::HeadDiverged);
        }
        confirm_document_head(self, phase, product, ()).map_err(|(_, denial)| denial)?;
        if expected != document.selection.lock().expected {
            return Err(Denial::HeadDiverged);
        }
        Ok(())
    }

    fn require_history_operation_ownership<H, O, I, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        admitted: &WorthQueryAdmittedApplicationOperation<S, O, I, E>,
    ) -> Result<(), Denial>
    where
        H: ApplicationHistoryBinding<S>,
    {
        if document.model.matches_operation(admitted) {
            return Ok(());
        }
        let scope = crate::domain_computation::primary_graph::product_operation::document_history::HistoryOperationScope::from_admitted(admitted);
        if !scope.belongs_to_model_runtime(&document.model) {
            return Err(Denial::ForeignRuntime);
        }
        let ownership = self
            .runtime
            .primary_provider
            .graph
            .layout
            .history_ownership(&installed.definition().parts().identity)
            .ok_or(Denial::InstalledBindingMismatch)?;
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let owner = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |native| {
                native.observe_model_owner(
                    &execution,
                    selected.product().relational_basis(),
                    ownership,
                    admitted.scope_entity_id(),
                )
            })
            .map_err(Denial::Ownership)?;
        if owner != Some(document.model.entity()) {
            return Err(Denial::UnownedOperationScope {
                subject: admitted.scope_entity_id(),
                model: document.model.entity(),
            });
        }
        Ok(())
    }
}
