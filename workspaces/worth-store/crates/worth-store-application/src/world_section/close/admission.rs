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
        // Funding and source validation finish before the actual final gates.
        if let Some(composition) = &self.composition {
            composition.preflight_home_retirement(request, self.home.relational().runtime())?;
        }
        if let Some(signal) = &self.standalone_signal {
            signal.preflight_home_retirement(request).map_err(|cause| {
                Denial::Bridge(
                    worth_runtime_bridge::facade::BridgeHomeRetirementDenial::AuthoredSignal(
                        Box::new(cause),
                    ),
                )
            })?;
        }
        let admission = match self.world.try_prepare_close_admission(request) {
            Ok(admission) => admission,
            Err(RuntimeWorldCloseDenial::AlreadyClosing) => return Ok(()),
            Err(cause) => return Err(Denial::World(RuntimeWorldServiceDenial::Denied(cause))),
        };
        if let Some(conditional) = self
            .composition
            .as_ref()
            .and_then(|composition| composition.conditional.as_ref())
        {
            let prepared = conditional
                .signal_owner()
                .prepare_home_stop(request)
                .map_err(|cause| {
                    Denial::Bridge(
                        worth_runtime_bridge::facade::BridgeHomeRetirementDenial::AuthoredSignal(
                            Box::new(cause),
                        ),
                    )
                })?;
            prepared.commit();
            // No fallible action may intervene before the actual final handoff.
            admission.begin();
        } else if let Some(signal) = &self.standalone_signal {
            let prepared = signal.prepare_home_stop(request).map_err(|cause| {
                Denial::Bridge(
                    worth_runtime_bridge::facade::BridgeHomeRetirementDenial::AuthoredSignal(
                        Box::new(cause),
                    ),
                )
            })?;
            prepared.commit();
            admission.begin();
        } else {
            return Err(Denial::WorldCompositionRetained);
        }
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
