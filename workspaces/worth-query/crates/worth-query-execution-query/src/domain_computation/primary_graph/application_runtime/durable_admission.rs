//! Operation-local requirements are checked before any supplied handler contact.
use super::super::application_entry::mutation::MutationHandlerExecutionDenial as Denial;
use super::super::WorthQueryOperationDurableCapability as Capability;
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial as DurableDenial;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn admit_directory_mutation<Binding: ApplicationMutationBinding<Schema>>(
        &self,
    ) -> Result<(), Denial> {
        if !self.runtime.product_runtime.is_directory_home() {
            return Ok(());
        }
        let installed = self
            .installed_schema
            .installed_mutation_binding::<Binding>()
            .map_err(Denial::DurableBindingAdmission)?;
        let contracts = installed.operation().contracts();
        let shape = self
            .runtime
            .product_runtime
            .admit_directory_original_shape(contracts, installed.correction_binding())
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
        if let Some(inbound) = shape.external_original_inbound() {
            self.admit_external_original_requirements(declared, workflow, inbound)?;
        }
        self.admit_contract_external_capabilities(contracts, declared)?;
        Ok(())
    }

    fn admit_contract_external_capabilities(
        &self,
        contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        declared: &[Capability],
    ) -> Result<(), Denial> {
        use worth_query_installation::facade::InstalledExternalEffectContract;
        let (needs_emission, needs_dispatch, needs_inbound) = match contracts.external_effect() {
            InstalledExternalEffectContract::Declared { inbound, .. } => {
                (true, true, inbound.is_some())
            }
            InstalledExternalEffectContract::None => {
                (!contracts.emissions().emissions().is_empty(), false, false)
            }
        };
        let required = [
            needs_emission.then_some(Capability::ExternalEmission),
            needs_dispatch.then_some(Capability::ExternalDispatch),
            needs_inbound.then_some(Capability::InboundCompletion),
        ];
        if !required
            .iter()
            .flatten()
            .all(|capability| declared.contains(capability))
        {
            return Err(Denial::DurableStateAdmission(
                DurableDenial::UndeclaredSupplierCapabilities,
            ));
        }
        Ok(())
    }

    fn admit_external_original_requirements(
        &self,
        declared: &[Capability],
        workflow: bool,
        inbound: bool,
    ) -> Result<(), Denial> {
        // Inbound completion custody is owed only where the contract declares it.
        let required = [
            Some(Capability::ExternalEmission),
            Some(Capability::ExternalDispatch),
            inbound.then_some(Capability::InboundCompletion),
        ];
        if workflow
            || !required
                .iter()
                .flatten()
                .all(|capability| declared.contains(capability))
        {
            return Err(Denial::DurableStateAdmission(
                DurableDenial::UnsupportedCorrectionCompensation,
            ));
        }
        Ok(())
    }
}
