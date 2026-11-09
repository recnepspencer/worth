//! Authored owner installation uses the same Native fresh-source admission.
use super::*;
use crate::lifecycle::{
    signal_services::RuntimeWorldSignalServices, RuntimeWorldAuthoredSignalAttachment,
};
pub type RuntimeWorldAuthoredHomeBuilder<D, I, E, Ctx, T> = RuntimeWorldOwnerBuilder<
    MissingRuntimeWorldInput,
    MissingRuntimeWorldInput,
    Arc<RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>>,
    MissingRuntimeWorldInput,
    RuntimeWorldBudgets,
    RuntimeWorldClock,
    ExecutionRequestPolicy,
>;
impl<D, I, E, Ctx, T> RuntimeWorldAuthoredHomeBuilder<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn admit_fresh_world_home(
        self,
        relational: RelationalOwnerServicePorts,
        bridge: RuntimeWorldCorrespondencePort,
        medium: Arc<dyn crate::durability::RuntimeWorldDurableMedium>,
        source: Box<dyn crate::durability::WorldCheckpointReadSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        admission: &worth_relational::facade::durability::FreshRelationalSourceAdmission,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::durability::WorldReadmissionDenial> {
        let binding =
            super::fresh_source::admit_source(medium, source, request, native, admission)?;
        let root = self.construct_home_root(relational, bridge, binding, Default::default())?;
        Ok(RuntimeWorldOwner::from_root(root))
    }
    pub fn readmit_world_home(
        self,
        relational: RelationalOwnerServicePorts,
        bridge: RuntimeWorldCorrespondencePort,
        prepared: &crate::durability::PreparedWorldHomeSource,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        native_recovery: &worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::durability::WorldReadmissionDenial> {
        let binding = prepared.binding();
        let recovered = prepared.recovered();
        super::signal_effects::readmit(
            &self.signal.source_port(),
            recovered,
            binding.binding(),
            request,
        )?;
        super::creation_effects::validate(
            native,
            native_recovery,
            recovered,
            binding.binding(),
            request,
        )?;
        let effects = super::associated_effects::readmit(
            native,
            native_recovery,
            recovered,
            binding.binding(),
            request,
        )?;
        let root = self.construct_home_root(
            relational,
            bridge,
            binding.clone(),
            recovered.frontiers.clone(),
        )?;
        super::reconcile::reconcile(
            &root,
            binding,
            recovered,
            native,
            native_recovery,
            effects,
            request,
        )?;
        Ok(RuntimeWorldOwner::from_root(root))
    }
    fn construct_home_root(
        self,
        relational: RelationalOwnerServicePorts,
        bridge: RuntimeWorldCorrespondencePort,
        binding: Arc<crate::durability::WorldDurableMediumBinding>,
        frontiers: crate::durability::WorldIdentityFrontiers,
    ) -> Result<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>, crate::durability::WorldReadmissionDenial>
    {
        use crate::durability::WorldReadmissionDenial as Denial;
        self.signal
            .admit_home(binding.binding())
            .map_err(|cause| Denial::AuthoredAttachment(Box::new(cause)))?;
        let execution =
            InstalledExecution::try_new(self.execution_authority, self.execution_policy)
                .map_err(Denial::ExecutionInstallation)?;
        let inputs = crate::lifecycle::owner_inputs::RuntimeWorldInstalledOwnerInputs {
            relational,
            signal: RuntimeWorldSignalServices::Authored(self.signal),
            bridge,
            budgets: self.budgets,
            clock: self.clock,
            execution,
        };
        let construction =
            crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract::for_home(
                binding.binding(),
                frontiers,
            )
            .map_err(Denial::IdentityExhaustion)?;
        let root = RuntimeWorldOwnerRoot::construct(inputs, construction, Some(binding))
            .map_err(Denial::IdentityExhaustion)?;
        Ok(root)
    }
}
