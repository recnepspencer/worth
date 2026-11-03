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
    pub(super) fn prepare_composite_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::ApplicationHomeCloseDenial> {
        use crate::ApplicationHomeCloseDenial as Denial;
        use worth_runtime_world::facade::{RuntimeWorldCloseDenial, RuntimeWorldServiceDenial};
        let Some(composition) = self.composition.as_ref() else {
            return Ok(());
        };
        // Funding and source validation finish before the actual final gates.
        composition.preflight_home_retirement(request, self.home.relational().runtime())?;
        let admission = match self.world.try_prepare_close_admission(request) {
            Ok(admission) => admission,
            Err(RuntimeWorldCloseDenial::AlreadyClosing) => return Ok(()),
            Err(cause) => return Err(Denial::World(RuntimeWorldServiceDenial::Denied(cause))),
        };
        let mut conditional = composition
            .conditional
            .try_write()
            .map_err(|_| Denial::WorldCompositionRetained)?;
        conditional
            .prepare_initial_empty_home_stop(request, self.home.relational().runtime())
            .map_err(Denial::Bridge)?;
        // No poll, allocation, callback, I/O or fallible action may intervene.
        admission.begin();
        Ok(())
    }
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
