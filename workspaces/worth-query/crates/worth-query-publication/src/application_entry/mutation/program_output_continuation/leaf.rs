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
    fn record_history_discovery(
        &mut self,
        origin: std::sync::Arc<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
    ) {
        let _ = (origin, index);
    }
    fn authorize_history(
        &self,
        _: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        _: ProgramOutputHistoryFrame<'_, '_, '_>,
    ) -> Result<(), WorthQueryRequiredOutputPreparationDenial> {
        Ok(())
    }
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
        _: Option<ProgramOutputHistoryFrame<'_, '_, '_>>,
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
    fn validate_history_group_outputs(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<(), worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial>{
        Ok(())
    }

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

    fn capture_history_root_recipes<
        H: worth_query_declaration::facade::application_history::ApplicationHistoryBinding<Schema>,
    >(
        document: &worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistory<'_, Schema, H>,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        profile: &worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputProfile,
        parent_demand: &ParentDemand,
    ) -> Result<(), crate::application_entry::WorthQueryHistoryOutputDenial> {
        let _ = (document, phase, application, profile, parent_demand);
        Ok(())
    }

    fn start_verified(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &ParentDemand,
        _: &worth_query_execution::publication_boundary::document_history::WorthQueryVerifiedHistoryParent<Schema, Program, ParentDemand>,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        _: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<Schema, Program>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(CompleteContinuation { open: true }))
    }

    fn start(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &ParentDemand,
        _: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        _: &ProgramOutputParentAuthority<Schema, Program, ParentDemand>,
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
