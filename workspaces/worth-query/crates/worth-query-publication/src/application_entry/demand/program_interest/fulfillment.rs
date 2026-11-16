//! The root recipe retains fulfillment through its actual admitted demand.
use super::*;
impl<S, P, D> WorthQueryApplicationProgramDemandHandle<S, P, D>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(in crate::application_entry) fn retain_fulfilled_root<R>(
        &self, phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        prepared: &worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        receipt: &worth_query_execution::facade::primary_graph::WorthQueryOutputSourceEvidence,
        settlement: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<S, D>>,
    ) -> Result<worth_query_execution::facade::primary_graph::WorthQueryFulfilledRequiredRoot, WorthQueryApplicationOutputDemandDenial>
    where R: worth_query_declaration::facade::application_program::ApplicationOutputGraphShape<S>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        <R::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<S>>::Binding:
            worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection<S, Demand = D>,
{
        self.admitted
            .ordinary()?
            .retain_fulfilled_required_root::<R>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                application,
                prepared,
                receipt,
                settlement.retained_settlement(),
            )
            .map_err(map_progress_denial)
    }
}
