//! A request borrows actual installed placement for its whole branded call.
use super::*;

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn with_execution_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(crate::lifecycle::RuntimeWorldExecutionPlacement<'owner>) -> R,
    ) -> Result<R, RuntimeWorldOwnerUnavailable> {
        let owner = self.service()?;
        Ok(body(owner.state.execution.placement()))
    }

    /// Passive custody for an existing public lease API. The actual World
    /// installs this authority; the handle does not keep the World alive.
    pub fn execution_authority_handle(
        &self,
    ) -> Result<Option<Arc<worth_execution::ExecutionAuthority>>, RuntimeWorldOwnerUnavailable>
    {
        Ok(self.service()?.state.execution.authority_handle())
    }
}
