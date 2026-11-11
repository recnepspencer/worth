//! Select the actual stored root or finish the new Home's original bootstrap.
use super::WorldReadmittedApplicationHome;
use crate::relational_preparation::HomeInitializationOrigin;
use worth_runtime_world::facade::*;

#[derive(Debug)]
pub enum ApplicationHomeQueryRootDenial {
    InitializationRequired,
    QueryCompositionNotAttached,
    Execution(worth_execution::WorkCeilingDenial),
    ExistingRoot(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    NativeBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    BootstrapRefused(WorldDurableBootstrapRefusal),
    RecoveryRequired(WorldDurableBootstrapRefusal),
    EvidenceRequired,
    Indeterminate,
    InitialSignalBranchRequired { observed: usize },
    Signal(worth_signal::facade::durability::SignalDurableSourceDenial),
}

impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn select_query_root(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ProductBranchObservation, ApplicationHomeQueryRootDenial> {
        use ApplicationHomeQueryRootDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        let existing = self.world.observation_port().observe_root_product_branch();
        if let Ok(root) = existing {
            return Ok(root);
        }
        match self.home.context.initialization {
            HomeInitializationOrigin::ConsumedNewHome => {}
            HomeInitializationOrigin::Available(
                worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts,
            ) => return Err(Denial::ExistingRoot(existing.unwrap_err())),
            HomeInitializationOrigin::Available(_) => return Err(Denial::InitializationRequired),
        }
        let port = self.world.publication_port();
        let mut outcome = port.advance_durable_bootstrap_root(request);
        if matches!(
            outcome,
            WorldDurableBootstrapOutcome::Refused(WorldDurableBootstrapRefusal::NoPendingBootstrap)
        ) {
            let composition = self
                .composition
                .as_ref()
                .ok_or(Denial::QueryCompositionNotAttached)?;
            let native = self.home.relational.runtime();
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .map_err(Denial::NativeBasis)?;
            let owner = composition
                .conditional
                .as_ref()
                .ok_or(Denial::QueryCompositionNotAttached)?
                .signal_owner();
            let source = owner.source_port();
            let branches = source.branches().map_err(Denial::Signal)?;
            if branches.len() != 1 {
                return Err(Denial::InitialSignalBranchRequired {
                    observed: branches.len(),
                });
            }
            let descriptor = source.observe(branches[0].0).map_err(Denial::Signal)?;
            let signal = source
                .readmit_descriptor(&descriptor)
                .map_err(Denial::Signal)?
                .basis()
                .clone();
            // This is the existing creation spelling, never a lookup guess for
            // an already recorded root, which was selected through its cell.
            outcome = port.begin_durable_bootstrap_root(
                request,
                ProductBranchCreationIntent::named("primary").expect("fixed valid creation name"),
                basis,
                signal,
            );
        }
        loop {
            match outcome {
                WorldDurableBootstrapOutcome::Performed(performed) => {
                    return Ok(performed.product_branch().clone())
                }
                WorldDurableBootstrapOutcome::Pending => {
                    request.consult().map_err(Denial::Execution)?;
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    outcome = port.advance_durable_bootstrap_root(request);
                }
                WorldDurableBootstrapOutcome::Refused(cause) => {
                    return Err(Denial::BootstrapRefused(cause))
                }
                WorldDurableBootstrapOutcome::RecoveryRequired(cause) => {
                    return Err(Denial::RecoveryRequired(cause))
                }
                WorldDurableBootstrapOutcome::EvidenceRequired => {
                    return Err(Denial::EvidenceRequired)
                }
                WorldDurableBootstrapOutcome::Indeterminate => return Err(Denial::Indeterminate),
            }
        }
    }
}
