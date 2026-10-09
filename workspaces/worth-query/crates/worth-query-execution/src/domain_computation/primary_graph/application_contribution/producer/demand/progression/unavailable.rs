//! Finish a failed original demand without scheduling or invoking its producer.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandSource, WorthQueryPreparedRequiredOutputSource,
};
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn finish_unavailable_output_demand<Family>(
        &self,
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
        if demand.runtime_authority != self.runtime.authority_identity().as_u64()
            || demand.schema_binding != self.installed_schema.binding_identity()
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
            self,
            demand,
            principal,
            scope,
            branch,
            true,
            disclosure,
            demand.installed_entry.edition,
        )?;
        let mut admission = self.demand_request_admission();
        let result = self.output_demands.finish_unavailable_source(
            prepared,
            interest,
            &self.primary_provider.graph.source_owner.invalidation_owner,
            &mut admission,
        )?;
        demand.close();
        Ok(result)
    }
}
