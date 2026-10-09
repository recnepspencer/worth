use super::*;

struct BranchContinuation<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    left: Option<Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>>,
    right: Option<Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>>,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
}

impl<Schema, Program> sealed::Continuation for BranchContinuation<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
}

impl<Left, Right> sealed::Factory for (Left, Right) {}

impl<Schema, Program, ParentDemand, Left, Right>
    ProgramOutputContinuationFactory<Schema, Program, ParentDemand> for (Left, Right)
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema> + Clone,
    Left: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<Schema, Program, ParentDemand>,
    Right: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<Schema, Program, ParentDemand>,
{
    fn start(
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        parent_authority: &std::sync::Arc<
            WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        >,
        minimum_observation: &crate::application_entry::WorthQueryApplicationReadObservation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(BranchContinuation {
            left: Some(Left::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                minimum_observation,
                request,
                controls,
            )?),
            right: Some(Right::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                minimum_observation,
                request,
                controls,
            )?),
            outputs: Vec::new(),
            work: ProgramOutputTraversalWork::default(),
        }))
    }
}

impl<Schema, Program> ProgramOutputContinuation<Schema, Program>
    for BranchContinuation<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn advance(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        for branch in [&mut self.left, &mut self.right] {
            let Some(active) = branch else {
                continue;
            };
            if let ProgramOutputContinuationProgress::Settled { outputs, work } =
                active.advance(application, request)?
            {
                self.outputs.extend(outputs);
                self.work.include(work);
                *branch = None;
            }
        }
        if self.left.is_some() || self.right.is_some() {
            return Ok(ProgramOutputContinuationProgress::Pending);
        }
        Ok(ProgramOutputContinuationProgress::Settled {
            outputs: std::mem::take(&mut self.outputs),
            work: self.work,
        })
    }
}
