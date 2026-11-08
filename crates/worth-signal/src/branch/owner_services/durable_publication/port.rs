//! A weak publication service never grants a memory-only canonical movement door.
use super::{SignalDurableAdvanceDenial, SignalDurableAdvanceTask, SignalDurablePublicationState};
use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};
pub struct SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) state: Weak<SignalDurablePublicationState<D, I, T>>,
    pub(super) application: PhantomData<fn() -> (E, Ctx)>,
}
impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn from_owner(state: &Arc<SignalDurablePublicationState<D, I, T>>) -> Self {
        Self {
            state: Arc::downgrade(state),
            application: PhantomData,
        }
    }
}
impl<D, I, E, Ctx, T> Clone for SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            application: PhantomData,
        }
    }
}
impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn resume_original(
        &self,
        original: worth_foundational::facade::SignalBranchAdvanceIdentity,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        let state = self
            .state
            .upgrade()
            .ok_or(SignalDurableAdvanceDenial::OwnerUnavailable)?;
        state.require_standalone()?;
        state.task(original)?;
        Ok(state.handle(original))
    }
}
