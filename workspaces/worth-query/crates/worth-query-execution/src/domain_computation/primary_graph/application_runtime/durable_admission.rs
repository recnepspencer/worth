//! Operation-local requirements are checked before any supplied handler contact.
use super::super::application_entry::mutation::MutationHandlerExecutionDenial as Denial;
use super::super::WorthQueryOperationDurableCapability as Capability;
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial as DurableDenial;
use worth_foundational::facade::DurableStateFamily as Family;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_directory_mutation<
        Binding: ApplicationMutationBinding<Schema>,
    >(
        &self,
    ) -> Result<(), Denial> {
        if !self.runtime.product_runtime.is_directory_home() {
            return Ok(());
        }
        let require = |family| {
            self.runtime
                .product_runtime
                .admit_directory_family(family)
                .map_err(Denial::DurableStateAdmission)
        };
        // The base ordinary path includes interrupted effects and graph integrity.
        for family in [
            Family::QueryIdempotencyBinding,
            Family::UnpublishedIdempotency,
            Family::RelationalPendingSettlement,
            Family::WorldAttemptProgress,
            Family::WorldUnpublishedCustody,
            Family::PendingApplicationPublication,
        ] {
            require(family)?;
        }
        let installed = self
            .installed_schema
            .installed_mutation_binding::<Binding>()
            .map_err(Denial::DurableBindingAdmission)?;
        self.runtime
            .product_runtime
            .admit_directory_correction_shape(
                installed.operation().contracts().aftermath(),
                installed.correction_binding(),
            )
            .map_err(Denial::DurableStateAdmission)?;
        let declared = self
            .mutation_handlers
            .declared_durable_capabilities::<Binding>()?
            .ok_or(Denial::DurableStateAdmission(
                DurableDenial::UndeclaredSupplierCapabilities,
            ))?;
        let require_capability = |capability| {
            self.runtime
                .product_runtime
                .admit_directory_capability(capability)
                .map_err(Denial::DurableStateAdmission)
        };
        for &capability in declared {
            require_capability(capability)?;
        }
        if Binding::REQUIRES_WORKFLOW_AUTHORITY
            || Binding::WORKFLOW_CONTROL
            || Binding::workflow_execution_requirements().is_applicable()
        {
            require_capability(Capability::WorkflowExecution)?;
        }
        if installed
            .operation()
            .body()
            .is_some_and(|body| body.external_emission().is_some())
        {
            require_capability(Capability::ExternalEmission)?;
        }
        if installed.correction_binding().is_some() {
            require_capability(Capability::RetainedCorrectionMaterial)?;
        }
        Ok(())
    }
}
