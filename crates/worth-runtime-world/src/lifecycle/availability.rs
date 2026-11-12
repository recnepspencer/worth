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
        // A borrowed pre-close cut holds close metadata on its caller's
        // thread. Refuse that explicit exclusion before observing the mutex;
        // ordinary readers share this gate and never exclude each other.
        let Ok(_ordinary_admission) = self.state.operation.call_exclusion.try_read() else {
            return false;
        };
        self.owner_is_present()
            && self.lifecycle_observation() == RuntimeWorldOwnerLifecycleObservation::Open
    }
}
