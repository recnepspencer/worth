use crate::history::{CompositeHistoryCatalogDenial, CompositeHistoryTraversal};
use crate::identity::CompositeCommitIdentity;
use std::num::NonZeroUsize;
pub(crate) trait RuntimeWorldInspectionService:
    crate::lifecycle::availability::RuntimeWorldAvailability
{
    fn bootstrap_is_complete(&self) -> bool;
    fn history_snapshot(&self) -> super::RuntimeWorldHistorySnapshot;
    fn retention_costs(&self) -> crate::retention::RetentionCostSnapshot;
    fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial>;

    fn inspect_retention(
        &self,
        key: &crate::inspection::RuntimeWorldRetentionKey,
    ) -> Result<
        Option<crate::inspection::RuntimeWorldRetentionEntry>,
        crate::inspection::RuntimeWorldRetentionInspectionDenial,
    >;

    fn recovery_page(
        &self,
        after: Option<&crate::inspection::RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<
        crate::inspection::RuntimeWorldRecoveryPage,
        crate::recovery::RuntimeWorldRecoveryDenial,
    >;

    fn recovery_snapshot(&self) -> crate::inspection::RuntimeWorldRecoverySnapshot;
    fn retention_snapshot(&self) -> crate::inspection::RuntimeWorldRetentionSnapshot;
}
