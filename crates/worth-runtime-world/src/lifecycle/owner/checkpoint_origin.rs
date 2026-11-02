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
        let origin = binding.take_checkpoint_origin()?;
        drop(close);
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
    ) -> Result<WorldCheckpointOriginRetirement, WorldCheckpointOriginRetirementDenial> {
        self.root.retire_checkpoint_origin()
    }
}
