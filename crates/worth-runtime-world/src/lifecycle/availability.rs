use super::{RuntimeWorldOwnerLifecycleObservation, RuntimeWorldOwnerRoot};

/// Private dispatch only for World capabilities; component bundles stay concrete.
pub(crate) trait RuntimeWorldAvailability {
    fn is_available(&self) -> bool;
}

impl<D, I, E, Ctx, T> RuntimeWorldAvailability for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn is_available(&self) -> bool {
        self.owner_is_present()
            && self.lifecycle_observation() == RuntimeWorldOwnerLifecycleObservation::Open
    }
}
