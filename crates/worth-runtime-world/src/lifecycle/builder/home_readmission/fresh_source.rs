//! Fresh admission requires the same Native birth and an actually empty Home cut.
use super::*;
use crate::durability::{
    WorldDurableMediumBinding, WorldDurableMediumDenial, WorldReadmissionDenial,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::FreshRelationalSourceAdmission, runtime::RelationalRuntime,
};

impl<D, I, E, Ctx, T> RuntimeWorldHomeBuilder<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Closed Home attachment door for its actual fresh Native admission.
    /// A selected checkpoint or any durable World record requires recovery.
    pub fn admit_fresh_world_home(
        self,
        relational: RelationalOwnerServicePorts,
        bridge: RuntimeWorldCorrespondencePort,
        medium: Arc<dyn crate::durability::RuntimeWorldDurableMedium>,
        source: Box<dyn crate::durability::WorldCheckpointReadSource>,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        admission: &FreshRelationalSourceAdmission,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, WorldReadmissionDenial> {
        request.consult().map_err(request_denial)?;
        admission
            .validate_fresh_source_runtime(request, native)
            .map_err(WorldReadmissionDenial::NativeFreshSource)?;
        source
            .read(request)
            .map_err(WorldReadmissionDenial::Medium)?
            .require_fresh_empty()?;
        // No owner is constructed, and no recovered effects are discovered,
        // until the original request and both actual source admissions agree.
        request.consult().map_err(request_denial)?;
        let binding = WorldDurableMediumBinding::new(medium, request)
            .map_err(WorldReadmissionDenial::Medium)?;
        let root = self.construct_home_root(relational, bridge, binding, Default::default())?;
        Ok(RuntimeWorldOwner {
            transferred_to_home_retirement: false,
            root: Arc::new(root),
        })
    }
}

fn request_denial(cause: worth_execution::WorkCeilingDenial) -> WorldReadmissionDenial {
    WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
}
