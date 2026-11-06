//! Store supplies the actual readmitted Native owner at this closed seam.
use super::super::RuntimeWorldOwnerRoot;
use super::*;
mod associated_effects;
mod construction;
mod fresh_source;
mod native_effects;
mod reconcile;

/// Home configuration lacks caller-supplied Relational and Bridge owners.
/// Store installs the concrete section and its actual bound Bridge once.
pub type RuntimeWorldHomeBuilder<D, I, E, Ctx, T> = RuntimeWorldOwnerBuilder<
    MissingRuntimeWorldInput,
    MissingRuntimeWorldInput,
    SignalOwnerServicePorts<D, I, E, Ctx, T>,
    SignalConditionalDefinitionPublicationPort<D, I, E, Ctx, T>,
    RuntimeWorldBudgets,
    RuntimeWorldClock,
    ExecutionRequestPolicy,
>;

impl<D, I, E, Ctx, T> RuntimeWorldHomeBuilder<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Named owner readmission/attachment door. Its callers are mechanically
    /// closed to Store's concrete Home stage; raw identities cannot attach.
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
        let effects = associated_effects::readmit(
            native,
            native_recovery,
            &recovered,
            binding.binding(),
            request,
        )?;
        let root = self.construct_home_root(
            relational,
            bridge,
            binding.clone(),
            recovered.frontiers.clone(),
        )?;
        reconcile::reconcile(
            &root,
            &binding,
            &recovered,
            native,
            native_recovery,
            effects,
            request,
        )?;
        Ok(RuntimeWorldOwner::from_root(root))
    }
}
