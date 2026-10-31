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
    /// The authentic owner borrow keeps this exact root alive through every
    /// return, including Busy/Stopped after partial retirement. No weak upgrade
    /// can become a last-root destructor on the request-bound close path.
    ///
    /// ```compile_fail,E0382
    /// use worth_runtime_world::facade::RuntimeWorldOwner;
    /// fn close_after_drop(owner: RuntimeWorldOwner<(), (), (), (), ()>,
    ///     request: &worth_execution::ExecutionRequest<'_, '_>) {
    ///     let port = owner.lifecycle_port();
    ///     drop(owner);
    ///     port.close(request, &owner).unwrap();
    /// }
    /// ```
    pub fn close<D, I, E, Ctx, T>(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &super::super::RuntimeWorldOwner<D, I, E, Ctx, T>,
    ) -> Result<
        super::super::RuntimeWorldCloseReport,
        super::super::RuntimeWorldServiceDenial<super::super::RuntimeWorldCloseDenial>,
    >
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        // Close alone may reach the SAME owner in Closing to resume its task.
        // All ordinary operations continue using service() and require Open.
        if self.owner.as_ptr() as *const () != Arc::as_ptr(&owner.root) as *const () {
            return Err(RuntimeWorldOwnerUnavailable::new().into());
        }
        RuntimeWorldLifecycleService::close(owner.root.as_ref(), request)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    /// Automatically retires changed, unprotected history interiors throughout
    /// this owner. Unchanged protected history is not traversed. The owner
    /// returns only identities actually retired; their order is unspecified.
    /// A live head, held observation/receipt, forming child or unconsumed
    /// publication delivery remains protected. Component pins release outside
    /// the catalog lock. This is distinct from scoped ancestry traversal below.
    pub fn retire_changed_history(
        &self,
    ) -> Result<
        Vec<crate::identity::CompositeCommitIdentity>,
        RuntimeWorldServiceDenial<HistoryReclamationDenial>,
    > {
        self.service()?
            .retire_changed_history()
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
