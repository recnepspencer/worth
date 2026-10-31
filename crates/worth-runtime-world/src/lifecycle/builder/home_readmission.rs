//! Store supplies the actual readmitted Native owner at this closed seam.
use super::super::RuntimeWorldOwnerRoot;
use super::*;

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
        medium: Box<dyn crate::durability::RuntimeWorldDurableMedium>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::durability::WorldReadmissionDenial> {
        use crate::durability::{
            WorldDurableMediumBinding, WorldReadmissionDenial as Denial, WorldRecoveredJournalState,
        };
        let binding = WorldDurableMediumBinding::new(medium);
        let source = binding.recover(request).map_err(Denial::Medium)?;
        let recovered = WorldRecoveredJournalState::readmit(source, binding.binding(), request)?;
        // Descriptions cannot stand in for Native's exact recovered-root and
        // recovered-publication capabilities. Refuse before constructing a
        // serving World when those owner admissions are still required.
        if !recovered.branches.is_empty()
            || !recovered.occurrences.is_empty()
            || !recovered.opens.is_empty()
        {
            return Err(Denial::NativeReadmissionRequired);
        }
        let execution =
            InstalledExecution::try_new(self.execution_authority, self.execution_policy)
                .map_err(Denial::ExecutionInstallation)?;
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
            super::super::owner::RuntimeWorldOwnerConstructionContract::readmitted_home(
                binding.binding(),
                recovered.frontiers,
            )
            .map_err(Denial::IdentityExhaustion)?;
        let root = RuntimeWorldOwnerRoot::construct(inputs, construction, Some(binding))
            .map_err(Denial::IdentityExhaustion)?;
        Ok(RuntimeWorldOwner {
            root: Arc::new(root),
        })
    }
}
