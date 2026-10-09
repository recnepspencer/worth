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
    fn advance(
        &mut self,
        _application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        if std::mem::take(&mut self.open) {
            Ok(ProgramOutputContinuationProgress::Settled {
                outputs: Vec::new(),
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
