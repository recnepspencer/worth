use super::*;

struct CompleteContinuation {
    open: bool,
}

impl sealed::Continuation for CompleteContinuation {}
impl sealed::Factory for ApplicationOutputLeaf {}

impl<Schema, Program> ProgramOutputContinuation<Schema, Program> for CompleteContinuation
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn close_read_request(
        &mut self,
        _phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        _application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial>
    {
        Ok(())
    }
    fn advance(
        &mut self,
        _phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        _application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        if std::mem::take(&mut self.open) {
            Ok(ProgramOutputContinuationProgress::Settled {
                outputs: Vec::new(),
                dependencies: None,
                work: ProgramOutputTraversalWork::default(),
            })
        } else {
            Err(WorthQueryRequiredOutputPreparationDenial::Closed)
        }
    }
}

impl<Schema, Program, ParentDemand> ProgramOutputContinuationFactory<Schema, Program, ParentDemand>
    for ApplicationOutputLeaf
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema> + Clone,
{
    fn unavailable_completion<Root>() -> Option<ProgramOutputUnavailableCompletion<Schema, Program, Root>>
    where
        Schema: 'static,
        Self: Sized,
        Root: ApplicationOutputGraphShape<Schema, Dependents = Self> + ApplicationRequiredOutputRoot,
        Binding<Schema, Root::RootConnection>: WorthQueryApplicationRequiredOutputConnection<Schema, Demand = ParentDemand>,
        SourceValue<Schema, ParentDemand>: WorthQueryApplicationProjection<Schema, SourceQuery<Schema, ParentDemand>> + Clone,
        <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Source<Schema, ParentDemand>>,
        <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    {
        Some(
            super::super::performed_outputs::WorthQueryApplicationProgramOutputHandle::<
                Schema,
                Program,
                Root,
            >::finish_unavailable_in_advancement,
        )
    }

    fn start(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &ParentDemand,
        _: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        _: &std::sync::Arc<WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>>,
        _: &crate::application_entry::WorthQueryApplicationReadObservation,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        _: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(CompleteContinuation { open: true }))
    }
}
