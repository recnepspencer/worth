//! The actual Home execution, input and owner construction shared by admission.
use super::*;
use crate::durability::{
    WorldDurableMediumBinding, WorldIdentityFrontiers, WorldReadmissionDenial,
};

impl<D, I, E, Ctx, T> RuntimeWorldHomeBuilder<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn construct_home_root(
        self,
        relational: RelationalOwnerServicePorts,
        bridge: RuntimeWorldCorrespondencePort,
        binding: Arc<WorldDurableMediumBinding>,
        frontiers: WorldIdentityFrontiers,
    ) -> Result<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>, WorldReadmissionDenial> {
        let execution =
            InstalledExecution::try_new(self.execution_authority, self.execution_policy)
                .map_err(WorldReadmissionDenial::ExecutionInstallation)?;
        let inputs = RuntimeWorldOwnerInputs::new(
            relational,
            self.signal,
            self.signal_definition_publication,
            bridge,
            self.budgets,
            self.clock,
            execution.request_policy(),
        )
        .with_execution(execution);
        let construction =
            super::super::super::owner::RuntimeWorldOwnerConstructionContract::for_home(
                binding.binding(),
                frontiers,
            )
            .map_err(WorldReadmissionDenial::IdentityExhaustion)?;
        RuntimeWorldOwnerRoot::construct(inputs, construction, Some(binding))
            .map_err(WorldReadmissionDenial::IdentityExhaustion)
    }
}
