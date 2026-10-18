use super::*;

struct BranchContinuation<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    left: Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>,
    right: Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>,
    complete: [bool; 2],
    outputs: Vec<ProgramOutputRecord>,
    dependencies: [Option<WorthQueryRetainedQueryDependencies>; 2],
    basis: crate::application_entry::WorthQueryApplicationReadObservation,
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
            left: Left::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                minimum_observation,
                request,
                controls,
            )?,
            right: Right::start(
                application,
                parent_demand,
                parent_settlement,
                parent_authority,
                minimum_observation,
                request,
                controls,
            )?,
            complete: [false, false],
            outputs: Vec::new(),
            dependencies: [None, None],
            basis: minimum_observation.retained_clone(),
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
    fn close_read_request(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial>
    {
        self.left.close_read_request(phase, application, branch)?;
        self.right.close_read_request(phase, application, branch)
    }
    fn advance(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        for (index, branch) in [&mut self.left, &mut self.right].into_iter().enumerate() {
            if self.complete[index] {
                continue;
            }
            if let ProgramOutputContinuationProgress::Settled {
                outputs,
                work,
                dependencies,
            } = branch.advance(phase, application, request)?
            {
                self.dependencies[index] = dependencies;
                self.outputs.extend(outputs);
                self.work.include(work);
                // A finished sibling can still supply an exact consumed
                // predecessor to the unfinished branch. Keep its owners.
                self.complete[index] = true;
            }
        }
        if !self.complete.iter().all(|complete| *complete) {
            return Ok(ProgramOutputContinuationProgress::Pending);
        }
        let dependencies = if self.dependencies.iter().all(Option::is_none) {
            None
        } else {
            // Keep both closed prefixes owned until the complete funded join seals.
            let combined = request
                .at(&self.basis)
                .combine_query_dependencies(phase, self.dependencies.iter().flatten())?;
            self.dependencies = [None, None];
            Some(combined)
        };
        Ok(ProgramOutputContinuationProgress::Settled {
            dependencies,
            outputs: std::mem::take(&mut self.outputs),
            work: self.work,
        })
    }
}
