//! Observe a completed publication without certifying reusable Current output.
use super::*;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryAcceptedOutputAuthority, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<S: ApplicationSchema + 'static, F: WorthQueryProducerOutputFamily<S>>
    WorthQueryAdmittedOutputDemand<S, F>
{
    pub(in crate::domain_computation::primary_graph) fn observe_own_completed_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<
        Option<std::sync::Arc<WorthQueryOutputDemandSettlement>>,
        WorthQueryOutputDemandDenial,
    > {
        // A joined cached completion is not a publication performed by this
        // original root. Reuse retains its actual final settlement instead.
        if self.producer_contacts_in_this_demand == 0 {
            return Ok(None);
        }
        let Some(interest) = &self.interest else {
            return Ok(None);
        };
        let Some(ready) = runtime.runtime.output_demands.peek_ready(interest)? else {
            return Ok(None);
        };
        if !matches!(
            &ready.authority,
            WorthQueryAcceptedOutputAuthority::Committed(_)
        ) {
            return Ok(None);
        }
        // The peek has released the registry lock. The existing constructor
        // reacquires this actual publication, retaining its original roles and
        // receipt. No Ready obligation is finished and no Current proof minted.
        WorthQueryOutputDemandSettlement::from_original_program_ready(
            execution,
            runtime,
            &ready,
            &self.selected.identity,
            F::IDENTITY,
            self.producer_contacts_in_this_demand,
        )
        .map(Some)
    }
}
