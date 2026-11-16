//! Operation-local requirements are checked before any supplied handler contact.
use super::super::application_entry::mutation::MutationHandlerExecutionDenial as Denial;
use super::super::WorthQueryOperationDurableCapability as Capability;
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial as DurableDenial;
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
        let installed = self
            .installed_schema
            .installed_mutation_binding::<Binding>()
            .map_err(Denial::DurableBindingAdmission)?;
        let shape = self
            .runtime
            .product_runtime
            .admit_directory_original_shape(
                installed.operation().contracts(),
                installed.correction_binding(),
            )
            .map_err(Denial::DurableStateAdmission)?;
        let declared = self
            .mutation_handlers
            .declared_durable_capabilities::<Binding>()?
            .ok_or(Denial::DurableStateAdmission(
                DurableDenial::UndeclaredSupplierCapabilities,
            ))?;
        let workflow = Binding::REQUIRES_WORKFLOW_AUTHORITY
            || Binding::WORKFLOW_CONTROL
            || Binding::workflow_execution_requirements().is_applicable();
        if shape.requires_external_original_custody() {
            self.admit_external_original_requirements(declared, workflow)?;
        }
        Ok(())
    }

    fn admit_external_original_requirements(
        &self,
        declared: &[Capability],
        workflow: bool,
    ) -> Result<(), Denial> {
        if workflow
            || ![
                Capability::ExternalEmission,
                Capability::ExternalDispatch,
                Capability::InboundCompletion,
            ]
            .iter()
            .all(|capability| declared.contains(capability))
        {
            return Err(Denial::DurableStateAdmission(
                DurableDenial::UnsupportedCorrectionCompensation,
            ));
        }
        Ok(())
    }
}
