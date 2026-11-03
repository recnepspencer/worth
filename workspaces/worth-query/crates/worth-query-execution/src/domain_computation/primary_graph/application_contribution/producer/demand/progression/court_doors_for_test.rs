//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> where
    Schema: ApplicationSchema + 'static, {
    #[doc(hidden)]
    pub fn advance_program_output_demand_for_test<Family>(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static, {
        Self::advance_program_output_demand::<Family>(self, execution, phase, demand, principal, request_scope, delivery_branch, disclosure)
    }
}
