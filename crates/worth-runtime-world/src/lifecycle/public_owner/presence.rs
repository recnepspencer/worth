//! Ordinary owner abandonment; authenticated consuming handoff disarms it.
use crate::lifecycle::RuntimeWorldOwnerRoot;
use std::sync::{Arc, Weak};
trait PresenceOwner: Send + Sync {
    fn release_public_presence(&self);
}
pub(in crate::lifecycle) struct PublicWorldPresence {
    owner: Weak<dyn PresenceOwner>,
    active: bool,
}
impl PublicWorldPresence {
    pub(super) fn new<D, I, E, Ctx, T>(root: &Arc<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>>) -> Self
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        let owner: Arc<dyn PresenceOwner> = root.clone();
        Self {
            owner: Arc::downgrade(&owner),
            active: true,
        }
    }
    pub(in crate::lifecycle) fn disarm(&mut self) {
        self.active = false;
    }
    pub(in crate::lifecycle) fn rearm(&mut self) {
        self.active = true;
    }
}
impl Drop for PublicWorldPresence {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Some(owner) = self.owner.upgrade() {
            owner.release_public_presence();
        }
    }
}
impl<D, I, E, Ctx, T> PresenceOwner for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn release_public_presence(&self) {
        self.release_public_owner();
    }
}
