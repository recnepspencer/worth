use super::*;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn advance_program_output<Demand>(
        &self,

        phase: &WorthQueryAdvancementPhase<'_>,

        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
    ) -> Result<WorthQueryProgramOutputAdvance<Schema, Program, Demand>, WorthQueryOutputDemandDenial>
    where
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        SourceValue<Schema, Demand>: 'static,
        SourceQuery<Schema, Demand>: 'static,
    {
        let phase_request = match phase.execution_request_for(&self.backing.runtime.product_runtime) {
            Ok(request) => request,
            Err(cause) => return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "output-demand-phase")),
        };
        let execution = &phase_request;

        self.backing
            .runtime
            .advance_program_output_demand(
                execution,
                phase,
                &mut demand.admitted,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
            .map_err(|mut denial| {
                if let Some(cleanup) = demand.admitted.take_managed_cleanup() {
                    denial = denial.with_managed_cleanup(cleanup);
                }
                denial
            })
            .map(|progress| match progress {
                WorthQueryOutputDemandAdvance::Pending => WorthQueryProgramOutputAdvance::Pending,
                WorthQueryOutputDemandAdvance::Settled(retained) => {
                    WorthQueryProgramOutputAdvance::Settled(WorthQuerySettledProgramOutput {
                        retained,
                        target_feature: demand.target_feature,
                        artifact: demand.artifact,
                        marker: std::marker::PhantomData,
                    })
                }
            })
    }
}
