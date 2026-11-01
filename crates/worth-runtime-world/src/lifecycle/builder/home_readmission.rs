//! Store supplies the actual readmitted Native owner at this closed seam.
use super::super::RuntimeWorldOwnerRoot;
use super::*;
mod associated_effects;
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
        medium: Arc<dyn crate::durability::RuntimeWorldDurableMedium>,
        source: Box<dyn crate::durability::WorldCheckpointReadSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        native_recovery: &worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::durability::WorldReadmissionDenial> {
        use crate::durability::{
            WorldDurableMediumBinding, WorldReadmissionDenial as Denial, WorldRecoveredJournalState,
        };
        let binding = WorldDurableMediumBinding::new(medium);
        let (recovered, selected_coverage) = WorldRecoveredJournalState::read_protected_source(
            source.as_ref(),
            binding.binding(),
            request,
        )?;
        binding
            .admit_recovered_checkpoint_coverage(selected_coverage)
            .map_err(Denial::Medium)?;
        if recovered.branches.len() > 1
            || recovered.dispositions.values().any(|disposition| {
                matches!(
                    disposition.disposition(),
                    crate::durability::RecoveredCustodyDisposition::Continued
                )
            })
        {
            return Err(Denial::OccurrenceReadmissionRequired);
        }
        for origin in recovered.branch_origins() {
            let artifact = origin.record();
            let crate::durability::WorldRecord::Branch {
                program_revision, ..
            } = &artifact.record
            else {
                unreachable!("branch frame")
            };
            if *program_revision != binding.binding().program_revision() {
                return Err(Denial::ProgramRevisionMismatch);
            }
        }
        if recovered
            .opens
            .values()
            .any(|open| open.program_revision != binding.binding().program_revision())
        {
            return Err(Denial::ProgramRevisionMismatch);
        }
        let effects = associated_effects::readmit(
            native,
            native_recovery,
            &recovered,
            binding.binding(),
            request,
        )?;
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
                recovered.frontiers.clone(),
            )
            .map_err(Denial::IdentityExhaustion)?;
        let root = RuntimeWorldOwnerRoot::construct(inputs, construction, Some(binding.clone()))
            .map_err(Denial::IdentityExhaustion)?;
        reconcile::reconcile(
            &root,
            &binding,
            &recovered,
            native,
            native_recovery,
            effects,
            request,
        )?;
        Ok(RuntimeWorldOwner {
            root: Arc::new(root),
        })
    }
}
