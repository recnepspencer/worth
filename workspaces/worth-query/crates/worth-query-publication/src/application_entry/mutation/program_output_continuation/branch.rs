use super::*;

struct BranchContinuation<'application, Schema>
where
    Schema: ApplicationSchema,
{
    left: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
    right: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
}

impl<'application, Schema> sealed::Continuation for BranchContinuation<'application, Schema> where
    Schema: ApplicationSchema
{
}

impl<Left, Right> sealed::Factory for (Left, Right) {}

impl<'application, Schema, Program, ParentDemand, Left, Right>
    ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand> for (Left, Right)
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
    Left: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>,
    Right: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>,
{
    fn start(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        parent_authority: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(BranchContinuation {
            left: Some(Left::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                request,
                controls,
            )?),
            right: Some(Right::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                request,
                controls,
            )?),
            outputs: Vec::new(),
            work: ProgramOutputTraversalWork::default(),
        }))
    }
}

impl<'application, Schema> ProgramOutputContinuation<'application, Schema>
    for BranchContinuation<'application, Schema>
where
    Schema: ApplicationSchema,
{
    fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        for branch in [&mut self.left, &mut self.right] {
            let Some(active) = branch else {
                continue;
            };
            if let ProgramOutputContinuationProgress::Settled { outputs, work } =
                active.advance(request)?
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
