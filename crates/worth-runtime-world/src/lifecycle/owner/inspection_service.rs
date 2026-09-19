use super::RuntimeWorldOwnerRoot;
use crate::history::{CompositeHistoryCatalogDenial, CompositeHistoryTraversal};
use crate::identity::CompositeCommitIdentity;
use crate::inspection::{RuntimeWorldHistorySnapshot, RuntimeWorldInspectionService};
use std::num::NonZeroUsize;
impl<D, I, E, Ctx, T> RuntimeWorldInspectionService for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn bootstrap_is_complete(&self) -> bool {
        *self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            == super::RuntimeWorldBootstrapState::Performed
    }
    fn history_snapshot(&self) -> RuntimeWorldHistorySnapshot {
        self.state.history.snapshot()
    }
    fn retention_costs(&self) -> crate::retention::RetentionCostSnapshot {
        self.state.retention.cost_snapshot()
    }
    fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial> {
        self.state.history.trace_ancestry(start, maximum)
    }

    fn inspect_retention(
        &self,
        key: &crate::inspection::RuntimeWorldRetentionKey,
    ) -> Result<
        Option<crate::inspection::RuntimeWorldRetentionEntry>,
        crate::inspection::RuntimeWorldRetentionInspectionDenial,
    > {
        self.state.retention.inspect_key(key)
    }

    fn recovery_page(
        &self,
        after: Option<&crate::inspection::RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<
        crate::inspection::RuntimeWorldRecoveryPage,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        let now = self.state.clock.now();
        self.state.recovery.page(after, maximum, now)
    }

    fn recovery_snapshot(&self) -> crate::inspection::RuntimeWorldRecoverySnapshot {
        self.state.recovery.snapshot()
    }
    fn retention_snapshot(&self) -> crate::inspection::RuntimeWorldRetentionSnapshot {
        self.state
            .retention
            .snapshot(self.state.publication_capacity.active())
    }
}
