//! SDK cleanup precedes retirement of Native history and its selected heads.
use super::{retained::*, *};

impl<S, P, R> WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn release_selected_branch(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        use std::ops::Bound::{Excluded, Unbounded};
        let execution = phase
            .checked_execution_request_for(self.application.runtime().product_runtime())
            .map_err(WorthQueryHistoryOutputDenial::Execution)?;
        let mut after = None;
        loop {
            // This registry contains only unfinished attempts or actual cleanup
            // tails. Completed empty slots are removed, so old replay metadata
            // never turns this bounded live-owner pass into an all-history scan.
            let next = {
                let registry = self
                    .slots
                    .try_lock()
                    .map_err(|_| WorthQueryHistoryOutputDenial::Busy)?;
                let units = 4 * u64::from(usize::BITS - registry.len().leading_zeros()) + 6;
                execution
                    .run_native_with_checkpoint(|context| {
                        if context.has_accounting() {
                            context.checkpoint(units)
                        } else {
                            context.poll()
                        }
                    })
                    .map_err(WorthQueryHistoryOutputDenial::Work)?
                    .0
                    .map_err(|stop| {
                        WorthQueryHistoryOutputDenial::Work(
                            worth_execution::WorkCeilingDenial::Stopped(stop),
                        )
                    })?;
                let entry = match after {
                    None => registry.first_key_value(),
                    Some(key) => registry.range((Excluded(key), Unbounded)).next(),
                };
                entry.map(|(key, slot)| (*key, std::sync::Arc::clone(slot)))
            };
            let Some((key, slot)) = next else {
                return Ok(());
            };
            after = Some(key);
            let mut state = slot.try_state()?;
            if state.selected
                && (state.empty()
                    || state
                        .original
                        .as_ref()
                        .is_some_and(|(actual, _)| *actual == branch))
            {
                state.dispose(phase, &self.application)?;
                drop(state);
                self.remove_slot(&key, &slot)?;
            }
        }
    }

    pub(super) fn drain(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        // No principal or permission is needed to release original custody.
        phase
            .check_live_for(self.application.runtime().product_runtime())
            .map_err(WorthQueryHistoryOutputDenial::Execution)?;
        let registry = self
            .slots
            .get_mut()
            .map_err(|_| WorthQueryHistoryOutputDenial::Busy)?;
        for slot in registry.values() {
            slot.try_state()?.dispose(phase, &self.application)?;
        }
        registry.clear();
        Ok(())
    }
}

impl<S, P, R> SlotState<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn dispose(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        if let Some(cleanup) = &mut self.read_cleanup {
            cleanup
                .close_in_advancement(phase, application.runtime())
                .map_err(Denial::Close)?;
        }
        self.read_cleanup = None;
        if let Some((outcome, cleanup)) = self.managed_cleanup.take() {
            if let Err(cleanup) = cleanup.retry_in_advancement(phase) {
                self.managed_cleanup = Some((outcome, cleanup));
                return Err(Denial::ManagedCleanup);
            }
        }
        if let (Some(tree), Some((branch, _))) = (&mut self.tree, &self.original) {
            tree.close_read_request(phase, application, *branch)
                .map_err(Denial::Close)?;
        }
        self.tree = None;
        self.settlement = None;
        self.source = None;
        self.original = None;
        Ok(())
    }
}
