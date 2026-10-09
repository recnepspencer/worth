use super::*;

impl<Schema, Intent, Program, Root>
    WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
    Root::Dependents:
        crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<Schema,
            Program,
            ProgramDemand<Schema, Root>,
        >,
    ProgramDemand<Schema, Root>: Clone,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Root>>,
    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    <<DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<
                Schema,
                <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query,
            > + Clone,
{
    pub fn start_required_outputs(
        self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<WorthQueryStartedRequiredOutputs<Schema, Intent, Program, Root>, WorthQueryRequiredOutputStartFailure<Schema, Intent, Program, Root>> {
        let Self { result, source } = self;
        let receipt = source.receipt.clone();
        match source.start_required_outputs(application, request, controls) {
            Ok(required_output) => Ok(WorthQueryStartedRequiredOutputs { receipt, result, required_output }),
            Err((source, denial)) => Err(WorthQueryRequiredOutputStartFailure { performed: Self { result, source }, denial }),
        }
    }
}
