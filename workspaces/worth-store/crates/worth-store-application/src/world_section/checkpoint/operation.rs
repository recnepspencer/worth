//! Only the actual composed Home issues its Native/World checkpoint operation.
use crate::{
    ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits,
    ApplicationHomeCheckpointOperation, WorldReadmittedApplicationHome,
};

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn checkpoint_operation(
        &self,
        limits: ApplicationHomeCheckpointLimits,
    ) -> Result<ApplicationHomeCheckpointOperation, Denial> {
        let world = self
            .world
            .checkpoint_port()
            .ok_or(Denial::WorldSectionRequired)?;
        Ok(ApplicationHomeCheckpointOperation::new(
            self.home.context.serving.clone(),
            self.home.context.checkpoint.clone(),
            world,
            self.signal_checkpoint.clone(),
            self.relational().complete_checkpoint_preparation(),
            limits,
        ))
    }
}
