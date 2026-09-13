#[path = "finalization.rs"]
mod finalization;

#[path = "creation/execution.rs"]
mod execution;

use crate::branch::{
    ProductBranchCreationIntent, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial,
};
use crate::lifecycle::{RuntimeWorldBranchCreationOutcome, RuntimeWorldPreparationService};
use crate::publication::RuntimeWorldCancellationToken;

use execution::{execute_creation, BranchCreationExecution, CreationDestination};

use super::super::RuntimeWorldOwnerRoot;

/// Create one product branch whose components are forked rather than reused.
/// The registry name, the branch identity and every bounded reservation are
/// taken before the first owner fork, so a denial past this point is never a
/// silent capacity failure.
pub(super) fn create_forked_branch<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    source: ProductBranchObservation,
    intent: ProductBranchCreationIntent,
    cancellation: &RuntimeWorldCancellationToken,
) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let name = intent.name().clone();
    let mut reservation = owner
        .state
        .branches
        .reserve_branch(owner.owner_identity(), name.clone())
        .map_err(super::map_registry_denial)?;
    let (branch, incarnation) = owner
        .issue_branch_identities(name)
        .map_err(|()| RuntimeWorldBranchAdmissionDenial::IdentityExhausted)?;
    let destination = CreationDestination {
        witness: reservation
            .bind_creation_destination(branch.clone(), incarnation)
            .map_err(super::map_registry_denial)?,
        branch: branch.clone(),
        incarnation,
    };

    let attempt = RuntimeWorldPreparationService::prepare_creation(
        owner,
        source,
        intent,
        cancellation,
        None,
    )?;
    match execute_creation(owner, attempt, &destination, cancellation) {
        BranchCreationExecution::NoEffect(denial) => Err(denial),
        BranchCreationExecution::ProductUnpublished(effects) => Ok(
            RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects),
        ),
        BranchCreationExecution::Settled {
            attempt,
            progress,
            successor_basis,
        } => finalization::install_forked_branch(
            owner,
            finalization::ForkedBranchInstallation {
                branch,
                lifecycle: incarnation,
                reservation,
                attempt,
                progress,
                successor_basis,
            },
            cancellation,
        ),
    }
}
