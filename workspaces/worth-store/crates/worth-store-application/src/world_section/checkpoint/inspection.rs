//! Inspect the actual attached World's checkpoint owner and acknowledged cut.
use crate::{
    ApplicationHomeCheckpointDenial, ApplicationHomeCheckpointInspection,
    ApplicationHomeCheckpointLimits,
};
use worth_execution::ExecutionRequest;

impl<D, I, E, Ctx, T> super::super::WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Scheduling metadata from actual acknowledged prefixes. Inspecting it
    /// neither selects a checkpoint nor proves that an owner has no state.
    pub fn inspect_checkpoint(
        &self,
        limits: ApplicationHomeCheckpointLimits,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpointInspection, ApplicationHomeCheckpointDenial> {
        let world = self
            .world
            .checkpoint_port()
            .ok_or(ApplicationHomeCheckpointDenial::WorldSectionRequired)?;
        crate::checkpoint::inspection::inspect(
            &self.home.context.serving,
            &self.home.context.checkpoint,
            &world,
            limits,
            request,
        )
    }
}
