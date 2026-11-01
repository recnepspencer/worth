//! SDK cleanup precedes retirement of Native history and its selected heads.
use super::{retained::*, *};

impl<S, P, R> WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
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
