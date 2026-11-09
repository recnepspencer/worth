//! Compiled dispatch retains the actual typed tree and its cleanup registry.
use super::*;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use worth_query_declaration::facade::application_query::ApplicationQueryIntent;

impl<S, P, R> OriginalTree<S> for WorthQueryHistoryWithOutputs<S, P, R>
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
    fn prepare_recording(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<recording::RecordingEntry<S>, WorthQueryHistoryOutputDenial> {
        recording::prepare_tree::<S, P, R>(self, phase)
    }
    fn has_original_attempt(&self, key: &[u8; 32]) -> Result<bool, WorthQueryHistoryOutputDenial> {
        match self.existing_slot(key)? {
            Some(slot) => Ok(slot.try_state()?.original.is_some()),
            None => Ok(false),
        }
    }
    fn profile(&self) -> &WorthQueryHistoryOutputProfile {
        &self.profile
    }
    fn runtime(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
    {
        self.application.runtime()
    }
    fn progress(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: [u8; 32],
        step: Step<'_, '_>,
    ) -> Result<Option<Arc<WorthQueryRetainedHistoryOutputTree>>, WorthQueryHistoryOutputDenial>
    {
        self.progress_forest(phase, request, key, step)
    }
    fn release(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.release_selected_branch(phase, branch)
    }
    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &[u8; 32],
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        if let Some(slot) = self.existing_slot(key)? {
            slot.try_state()?.dispose(phase, &self.application)?;
            self.remove_slot(key, &slot)?;
        }
        Ok(())
    }
    fn selected(&self, phase: &WorthQueryAdvancementPhase<'_>, key: &[u8; 32]) {
        if let Ok(Some(slot)) = self.existing_slot(key)
            && let Ok(mut state) = slot.try_state()
        {
            state.selected = true;
            let disposed = state.dispose(phase, &self.application).is_ok();
            drop(state);
            if disposed {
                let _ = self.remove_slot(key, &slot);
            }
        }
    }
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        self.drain(phase)
    }
}
