//! One genuine ordinary program-output handle and its refusal-preserving cleanup.
use super::*;
mod verified;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use crate::application_entry::WorthQueryOutputDemandControls;
use crate::application_entry::{
    WorthQueryApplicationProgramOutputHandle, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationProgramOutputSettlement,
};
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;
struct OriginalRecordingTree<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    demand: Demand<S, R>,
    controls: WorthQueryOutputDemandControls,
    tree: Option<HistoryOutputTraversal<S, P, R>>,
    settlement: Option<WorthQueryApplicationProgramOutputSettlement<RootQuery<S, R>>>,
    retained: Option<Arc<WorthQueryRetainedRecordingOutputTree>>,
    read_cleanup: Option<worth_query_execution::facade::primary_graph::WorthQueryOutputReadCleanup>,
    managed_cleanup: Option<(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted,
        worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup,
    )>,
    terminal: bool,
    // Native tree, cleanup and readonly evidence are destroyed before this runtime.
    application: WorthQueryProgramApplicationRuntime<S, P>,
}
pub(in crate::application_entry::history::outputs::forest) fn prepare<S, P, R>(
    template: &WorthQueryHistoryWithOutputs<S, P, R>,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<RecordingEntry<S>, WorthQueryHistoryOutputDenial>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    let execution = phase
        .checked_execution_request_for(template.application.runtime().product_runtime())
        .map_err(WorthQueryHistoryOutputDenial::Execution)?;
    let memory = funding::reserve(
        &execution,
        std::mem::size_of::<OriginalRecordingTree<S, P, R>>(),
    )?;
    Ok(RecordingEntry {
        tree: Box::new(OriginalRecordingTree::<S, P, R> {
            demand: template.demand.clone(),
            controls: template.controls,
            tree: None,
            settlement: None,
            retained: None,
            read_cleanup: None,
            managed_cleanup: None,
            terminal: false,
            application: template.application.clone(),
        }),
        _memory: memory,
    })
}
impl<S, P, R> RecordingTree<S> for OriginalRecordingTree<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    fn runtime(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
    {
        self.application.runtime()
    }
    fn advance(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<Option<Arc<WorthQueryRetainedRecordingOutputTree>>, WorthQueryHistoryOutputDenial>
    {
        use WorthQueryHistoryOutputDenial as Denial;
        if self.terminal {
            return Err(Denial::NeedsDisposal);
        }
        if let Some(tree) = &self.retained {
            return Ok(Some(Arc::clone(tree)));
        }
        if self.tree.is_none() {
            let observation = request.retain_read(phase)
                .map_err(|cause| Denial::Preparation(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ReadObservation(cause)))?;
            if let Some(verified) = self
                .application
                .verify_recording_output_tree::<R>(
                    phase,
                    &program_publication_access(),
                    origin,
                    index,
                    &observation.retained,
                )
                .map_err(Denial::RecordingQualification)?
            {
                self.retained = Some(Arc::clone(&verified));
                return Ok(Some(verified));
            }
            if self.try_start_verified(phase, request, authority)? {
                return self.progress_verified(phase, request, origin, index, authority);
            }
            self.application.validate_initial_program_source::<R>(phase, &program_publication_access(), origin.observation(), origin.candidate())
                .map_err(|cause| Denial::Preparation(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::DemandExecution(cause)))?;
            let (root, original) = request
                .at(&observation)
                .demand(self.demand.clone())
                .controls(self.controls)
                .start_for_recording::<P, R>(phase, &self.application, origin, index)
                .map_err(|cause| {
                    Denial::Preparation(
                        crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                            cause,
                        ),
                    )
                })?;
            self.tree = Some(HistoryOutputTraversal::Original(
                WorthQueryApplicationProgramOutputHandle::new_initial(
                    crate::application_entry::WorthQueryApplicationReadObservation::new(original),
                    root,
                    self.demand.clone(),
                    self.controls,
                ),
            ));
        }
        if self
            .tree
            .as_ref()
            .and_then(HistoryOutputTraversal::verified)
            .is_some()
        {
            return self.progress_verified(phase, request, origin, index, authority);
        }
        if self.settlement.is_none() {
            match self
                .tree
                .as_mut()
                .and_then(HistoryOutputTraversal::original_mut)
                .expect("original tree")
                .advance_in_advancement(phase, &self.application, request)
            {
                Ok(
                    WorthQueryApplicationProgramOutputProgress::Pending
                    | WorthQueryApplicationProgramOutputProgress::Progressed,
                ) => return Ok(None),
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(settlement)) => {
                    self.settlement = Some(settlement)
                }
                Err(mut cause) => {
                    self.read_cleanup = cause.take_read_cleanup();
                    self.managed_cleanup = cause
                        .take_managed_cleanup()
                        .map(|cleanup| cleanup.into_parts());
                    self.terminal = self.read_cleanup.is_some() || self.managed_cleanup.is_some()
                        || cause.recovery_posture() == crate::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    return Err(Denial::Preparation(cause));
                }
            }
        }
        self.tree
            .as_mut()
            .and_then(HistoryOutputTraversal::original_mut)
            .expect("original tree")
            .drain_read_request(phase, &self.application, origin.candidate())
            .map_err(Denial::Close)?;
        let (root, dependents, dependencies) = self
            .settlement
            .as_mut()
            .expect("actual completion")
            .original_tree_parts();
        let retained = self
            .application
            .retain_original_recording_output_tree::<R>(
                phase,
                &program_publication_access(),
                origin,
                index,
                root,
                dependents,
                dependencies,
            )
            .map_err(Denial::Qualification)?;
        self.retained = Some(Arc::clone(&retained));
        Ok(Some(retained))
    }
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        if let Some(cleanup) = &mut self.read_cleanup {
            cleanup
                .close_in_advancement(phase, self.application.runtime())
                .map_err(Denial::Close)?;
        }
        self.read_cleanup = None;
        if let Some((outcome, cleanup)) = self.managed_cleanup.take()
            && let Err(cleanup) = cleanup.retry_in_advancement(phase)
        {
            self.managed_cleanup = Some((outcome, cleanup));
            return Err(Denial::ManagedCleanup);
        }
        if let Some(tree) = &mut self.tree {
            match tree {
                HistoryOutputTraversal::Original(tree) => {
                    tree.close_read_request(phase, &self.application, branch)
                }
                HistoryOutputTraversal::Verified(tree) => {
                    tree.continuation
                        .close_read_request(phase, &self.application, branch)
                }
            }
            .map_err(Denial::Close)?;
        }
        self.tree = None;
        self.retained = None;
        self.settlement = None;
        Ok(())
    }
}
