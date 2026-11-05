//! Finish a failed original demand without scheduling or invoking its producer.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandSource, WorthQueryPreparedRequiredOutputSource,
};
pub(in crate::domain_computation::primary_graph) trait UnavailableScopedOps<Schema: ApplicationSchema + 'static>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> {
fn finish_unavailable_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        phase
            .execution_request_for(self.product_runtime())
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "foreign advancement phase",
                )
            })?;
        if demand.runtime_authority != self.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema().binding_identity()
            || demand.settled
            || demand.settled_at_observation.is_some()
            || demand.unpublished_selected_checkpoint.is_some()
            || !demand.required_continuations.is_empty()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                Family::IDENTITY,
            ));
        }
        let interest = demand
            .interest
            .as_ref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, Family::IDENTITY))?;
        // Same exact query/request-affinity and installed-edition owner as advance;
        // validation consumes disclosure only and never enters producer execution.
        validate_disclosure(
            execution,
            self.application_contribution_host(),
            demand,
            principal,
            scope,
            branch,
            true,
            disclosure,
            demand.installed_entry.edition,
        )?;

        let result = self.output_production().demands().finish_unavailable_source(
            prepared,
            interest,
            &self
                .provider_graph()
                .source_owner
                .invalidation_owner,
        )?;
        demand.close();
        Ok(result)
    }
}

impl<Schema: ApplicationSchema + 'static, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> UnavailableScopedOps<Schema> for HostRuntime {}
