//! Drive the original typed successor checkpoint under fresh invocation authority.
use super::*;
impl<Schema, Family> TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    pub(super) fn advance_original_checkpoint(
        &mut self,

        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let execution = phase
            .execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "required-checkpoint-phase",
                )
            })?;
        self.authorize_source_in_current_invocation(
            &execution, runtime, principal, request, invocation,
        )?;
        let demand = self
            .demand
            .as_mut()
            .expect("installed successor retains its typed demand");
        if demand.unpublished_selected_checkpoint.is_some() {
            return Ok(false);
        }
        let interest = demand
            .interest
            .as_ref()
            .expect("installed successor retains its real Interest");
        let prepared = runtime
            .output_production
            .demands()
            .prepare_selected_checkpoint_finish(interest)?;
        let Some((finish, claim, checkpoint)) = prepared.claim()? else {
            return Ok(false);
        };
        runtime.advance_selected_output_checkpoint(
            &execution,
            phase,
            &demand.selected.identity,
            claim,
            checkpoint,
            request,
            &mut demand.unpublished_selected_checkpoint,
            finish,
            principal,
            invocation,
        )
    }
}
