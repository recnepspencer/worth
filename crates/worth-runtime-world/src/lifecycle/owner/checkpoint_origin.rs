//! Explicit post-close retirement of the original checkpoint recipe's custody.
use super::RuntimeWorldOwnerRoot;
use crate::durability::{WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial};
use crate::lifecycle::{close::RuntimeWorldCloseState, RuntimeWorldOwner};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn retire_checkpoint_origin(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial> {
        use WorldCheckpointOriginRetirementDenial as Denial;
        let close = self.state.close.try_lock().map_err(|_| Denial::Busy)?;
        match close.state() {
            RuntimeWorldCloseState::Open | RuntimeWorldCloseState::Closing => {
                return Err(Denial::OwnerNotClosed);
            }
            RuntimeWorldCloseState::Closed => {}
        }
        let binding = self.state.durable.as_ref().ok_or(Denial::NotHomeBound)?;
        let task = close.retirement_task().map_err(Denial::RecoveredCustody)?;
        drop(close);
        crate::lifecycle::close::retire_recovered(&self.state.recovery, &task, request, native)
            .map_err(Denial::RecoveredCustody)?;
        let origin = binding.take_checkpoint_origin()?;
        Ok(origin.retire())
    }
}

impl<D, I, E, Ctx, T> RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Release only the original recipe's basis obligation after World close.
    /// Home close orders this before the Native owner's final retirement.
    pub fn retire_checkpoint_origin(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial> {
        self.root.retire_checkpoint_origin(request, native)
    }
}
