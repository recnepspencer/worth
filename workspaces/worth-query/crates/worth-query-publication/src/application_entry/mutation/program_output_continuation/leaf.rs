use super::*;

struct CompleteContinuation {
    open: bool,
}

impl sealed::Continuation for CompleteContinuation {}
impl sealed::Factory for ApplicationOutputLeaf {}

impl<'application, Schema> ProgramOutputContinuation<'application, Schema> for CompleteContinuation
where
    Schema: ApplicationSchema,
{
    fn advance(
        &mut self,
        _: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
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

impl<'application, Schema, Program, ParentDemand>
    ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>
    for ApplicationOutputLeaf
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn start(
        _: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &ParentDemand,
        _: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        _: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        _: &crate::application_entry::WorthQueryApplicationReadObservation,
        _: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        _: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(CompleteContinuation { open: true }))
    }
}
