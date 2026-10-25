use super::super::ports::RuntimeWorldLifecycleService;
use super::super::RuntimeWorldOwnerUnavailable;
use crate::history::HistoryReclamationDenial;
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
    /// Releases the history a closed branch alone needed, from `from` (its
    /// retired head) toward the root, past its fork point into inherited
    /// history nothing live reaches. A commit a live product head or a live
    /// descendant reaches ends the release: that owner keeps the rest. A
    /// commit only a held read, receipt, forming child or unconsumed delivery
    /// needs is reported as the commit to resume from once it is let go.
    pub fn release_retired_branch_history(
        &self,
        from: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        crate::history::RetiredBranchHistoryRelease,
        RuntimeWorldServiceDenial<HistoryReclamationDenial>,
    > {
        self.service()?
            .release_retired_branch_history(from)
            .map_err(RuntimeWorldServiceDenial::Denied)
    }
    /// The commits of `candidates` history still retains, in their given
    /// order. Retired and released commits never return.
    pub fn retained_commits(
        &self,
        candidates: &[crate::identity::CompositeCommitIdentity],
    ) -> Result<Vec<crate::identity::CompositeCommitIdentity>, RuntimeWorldOwnerUnavailable> {
        Ok(self.service()?.retained_commits(candidates))
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
