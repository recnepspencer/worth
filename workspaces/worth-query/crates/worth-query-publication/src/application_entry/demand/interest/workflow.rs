use super::*;

impl<'application, 'principal, 'scope, Schema, Demand>
    WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn start_for_workflow<Spec>(
        self,
        workflow: worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary<'application, Schema, Spec>,
    ) -> Result<
        (
            WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
            Demand,
            WorthQueryOutputDemandControls,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Spec: worth_query_declaration::facade::application_program::ApplicationWorkflowSpec<
            Schema = Schema,
        >,
    {
        let application = self.application;
        let scope = self.scope;
        application
            .with_application_advancement(scope, |phase| {
                if !std::ptr::eq(self.application, workflow.runtime()) {
                    return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
                }
                let source = self.query_source(&phase)?.into_output_demand_source();
                let limits = self.resolved_limits();
                let controls = WorthQueryOutputDemandControls::from_limits(limits);
                let admitted = workflow
                    .admit_workflow_assessment_output(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        source,
                        limits,
                    )
                    .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
                Ok((admitted, self.demand, controls))
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }
}
