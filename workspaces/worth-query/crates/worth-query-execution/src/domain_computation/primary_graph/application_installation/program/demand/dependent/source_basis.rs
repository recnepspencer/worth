//! Original retained continuation bounds shared by read and demand admission.
use super::*;
use crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQuerySelectedProductOperation,
};

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub(in crate::domain_computation::primary_graph::application_installation::program::demand) fn select_dependent_output_basis<
        D,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        parent: &WorthQuerySettledProgramOutput<S, P, D>,
        basis: &WorthQueryApplicationReadObservation,
        minimum: &WorthQueryApplicationReadObservation,
    ) -> Result<WorthQuerySelectedProductOperation<'_, S>, WorthQueryOutputDemandDenial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
    {
        use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
        let parent = parent.retained.retained_read();
        let retained = self
            .runtime()
            .application_runtime()
            .select_application_read_observation_with_request(execution, basis);
        if let Err(cause @ Branch::Stopped(_)) = retained {
            return Err(WorthQueryOutputDemandDenial::product_selection(
                cause,
                "dependent basis",
            ));
        }
        let minimum_retained = retained.as_ref().ok().map(|_| {
            self.runtime()
                .application_runtime()
                .select_application_read_observation_with_request(execution, minimum)
        });
        if let Some(Err(cause @ Branch::Stopped(_))) = &minimum_retained {
            return Err(WorthQueryOutputDemandDenial::product_selection(
                *cause,
                "dependent minimum observation",
            ));
        }
        if retained.is_err()
            || minimum_retained
                .as_ref()
                .is_some_and(|basis| basis.is_err())
            || basis.branch_identity() != parent.branch_identity()
            || minimum.branch_identity() != parent.branch_identity()
            || basis.branch_incarnation() != parent.branch_incarnation()
            || minimum.branch_incarnation() != parent.branch_incarnation()
            || basis.selected_commit().ordinal() < parent.selected_commit().ordinal()
            || basis.selected_commit().ordinal() < minimum.selected_commit().ordinal()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent observation must be owned by this program and follow its settled parent and source publication on the same product branch",
            ));
        }
        Ok(retained.expect("validated dependent basis"))
    }
}
