use super::super::ports::RuntimeWorldLifecycleService;
use super::super::RuntimeWorldOwnerUnavailable;
use crate::history::{
    CompositeHistoryReclamationRequest, HistoryReclamationDenial, HistoryReclamationOutcome,
};
use crate::lifecycle::RuntimeWorldServiceDenial;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's lifecycle service.
#[derive(Clone)]
pub struct RuntimeWorldLifecyclePort {
    owner: Weak<dyn RuntimeWorldLifecycleService + Send + Sync>,
}
impl RuntimeWorldLifecyclePort {
    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldLifecycleService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldLifecycleService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn bootstrap_root(
        &self,
        intent: crate::branch::RuntimeWorldBootstrapIntent,
    ) -> Result<crate::branch::RuntimeWorldBootstrapOutcome, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.bootstrap_root(intent))
    }
    pub fn close(
        &self,
    ) -> Result<
        super::super::RuntimeWorldCloseReport,
        super::super::RuntimeWorldServiceDenial<super::super::RuntimeWorldCloseDenial>,
    > {
        self.service()?
            .close()
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    pub fn reclaim_history(
        &self,
        request: CompositeHistoryReclamationRequest,
    ) -> Result<HistoryReclamationOutcome, RuntimeWorldServiceDenial<HistoryReclamationDenial>>
    {
        self.service()?
            .reclaim_history(request)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
    /// Retires every commit behind `keep` that nothing protects: no product
    /// head, second child, observation, traversal, receipt or unconsumed
    /// delivery. The oldest kept commit becomes the history base, and a
    /// retired commit between kept ones is spliced out of the ancestry, so
    /// a traversal skips it and reports that it crossed retired history.
    /// Inspecting or continuing ancestry past the base fails closed. Returns
    /// the retired commits, oldest first.
    pub fn retire_unprotected_history(
        &self,
        keep: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        Vec<crate::identity::CompositeCommitIdentity>,
        RuntimeWorldServiceDenial<HistoryReclamationDenial>,
    > {
        self.service()?
            .retire_unprotected_history(keep)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
    pub fn reclaim_retention(
        &self,
        keys: &[crate::inspection::RuntimeWorldRetentionKey],
        maximum: usize,
    ) -> Result<
        crate::retention::RetentionReclamationReport,
        RuntimeWorldServiceDenial<crate::inspection::RuntimeWorldRetentionInspectionDenial>,
    > {
        self.service()?
            .reclaim_retention(keys, maximum)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
}
