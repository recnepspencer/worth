use crate::history::{CompositeHistoryCatalogDenial, CompositeHistoryTraversal};
use crate::identity::CompositeCommitIdentity;
use std::num::NonZeroUsize;
pub(crate) trait RuntimeWorldInspectionService:
    crate::lifecycle::availability::RuntimeWorldAvailability
{
    fn observe_program_activation_history(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::WorldProgramActivationHistory,
        crate::durability::WorldReadmissionDenial,
    >;
    fn history_snapshot(&self) -> super::RuntimeWorldHistorySnapshot;
    fn retention_costs(&self) -> crate::retention::RetentionCostSnapshot;
    fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial>;

    fn performed_publication_page(
        &self,
        after: Option<&crate::history::RuntimeWorldPublicationCursor>,
        maximum: NonZeroUsize,
    ) -> Result<crate::history::RuntimeWorldPublicationPage, CompositeHistoryCatalogDenial>;

    fn publication_frontier_is_current(
        &self,
        frontier: &crate::history::RuntimeWorldPublicationFrontier,
    ) -> Result<bool, CompositeHistoryCatalogDenial>;

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
