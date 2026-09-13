use crate::history::{CompositeHistoryCatalogDenial, CompositeHistoryTraversal};
use crate::identity::CompositeCommitIdentity;
use crate::inspection::RuntimeWorldInspectionService;
use crate::lifecycle::{RuntimeWorldOwnerUnavailable, RuntimeWorldServiceDenial};
use std::num::NonZeroUsize;
use std::sync::{Arc, Weak};
/// Weak access to bounded, read-only World inspection.
#[derive(Clone)]
pub struct RuntimeWorldInspectionPort {
    owner: Weak<dyn RuntimeWorldInspectionService + Send + Sync>,
}
impl RuntimeWorldInspectionPort {
    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldInspectionService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldInspectionService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() || !service.bootstrap_is_complete() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn history_snapshot(
        &self,
    ) -> Result<crate::inspection::RuntimeWorldHistorySnapshot, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.history_snapshot())
    }
    /// Cumulative registry work; concurrent attempts may contribute.
    pub fn retention_costs(
        &self,
    ) -> Result<crate::retention::RetentionCostSnapshot, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.retention_costs())
    }
    pub fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, RuntimeWorldServiceDenial<CompositeHistoryCatalogDenial>>
    {
        self.service()?
            .trace_ancestry(start, maximum)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }

    pub fn inspect_retention(
        &self,
        key: &crate::inspection::RuntimeWorldRetentionKey,
    ) -> Result<
        Option<crate::inspection::RuntimeWorldRetentionEntry>,
        RuntimeWorldServiceDenial<crate::inspection::RuntimeWorldRetentionInspectionDenial>,
    > {
        self.service()?
            .inspect_retention(key)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }

    pub fn recovery_page(
        &self,
        after: Option<&crate::inspection::RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<
        crate::inspection::RuntimeWorldRecoveryPage,
        RuntimeWorldServiceDenial<crate::recovery::RuntimeWorldRecoveryDenial>,
    > {
        self.service()?
            .recovery_page(after, maximum)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }

    pub fn recovery_snapshot(
        &self,
    ) -> Result<crate::inspection::RuntimeWorldRecoverySnapshot, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.recovery_snapshot())
    }
    pub fn retention_snapshot(
        &self,
    ) -> Result<crate::inspection::RuntimeWorldRetentionSnapshot, RuntimeWorldOwnerUnavailable>
    {
        Ok(self.service()?.retention_snapshot())
    }
}
