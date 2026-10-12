//! Enter one caller pass with its already prepared source.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// `disclosure` answers `None` once the caller has no further source to
    /// disclose in this call. A caller that reads its retained source again
    /// retries after source replacement or upstream completion; one that handed over its
    /// only disclosure answers `Pending` and discloses on its next advance.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn advance_output_demand_with_prepared_source<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        mut disclosure: impl FnMut(
            &Observed<Schema, Family>,
            &mut InvalidationEditAdmission,
        ) -> Result<
            Option<Disclosed<Schema, Family>>,
            WorthQueryOutputDemandDenial,
        >,
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let mut pass = AdvancePass::Initial;
        loop {
            let result = self.advance_caller_pass(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                &mut disclosure,
                pass,
                performed,
                commit_authority.clone(),
                request_admission,
            );
            crate::domain_computation::primary_graph::application_contribution::request_execution::record_caller_pass_for_test(&result);
            match result {
                Ok(AdvancePassProgress::Answer(answer)) => {
                    if matches!(&answer, WorthQueryOutputDemandAdvance::Settled(_)) {
                        demand.required_continuations.requested.clear();
                    }
                    return Ok(answer);
                }
                Ok(AdvancePassProgress::Continue(next)) => pass = next,
                Err(stop) => {
                    demand.required_continuations.requested.clear();
                    return Err(stop);
                }
            }
        }
    }
}
