//! Observe genuine World admission, then release every guard before callbacks.
use super::super::WorldReadmittedApplicationHome;
use worth_execution::ExecutionRequest;

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// This observation does not seal. The actual close reacquires and rechecks
    /// the SAME owner gates after composite owners finish their preflight.
    pub fn preflight_world_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::ApplicationHomeCloseDenial> {
        self.world
            .try_preflight_close_admission(request)
            .map_err(|cause| {
                crate::ApplicationHomeCloseDenial::World(
                    worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(cause),
                )
            })
    }
}
