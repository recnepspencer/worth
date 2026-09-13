#[cfg(test)]
#[path = "finalization/custody_tests.rs"]
mod custody_tests;
#[path = "finalization/install.rs"]
mod install;
#[path = "finalization/state.rs"]
mod state;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::RuntimeWorldBranchAdmissionDenial;
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};
use crate::lifecycle::RuntimeWorldBranchCreationOutcome;
use crate::publication::{CompositeAttemptProgress, ReservedBranchCreationAttempt};

use super::super::super::RuntimeWorldOwnerRoot;

pub(super) struct ForkedBranchInstallation {
    pub(super) branch: ProductBranchIdentity,
    pub(super) lifecycle: ProductBranchIncarnation,
    pub(super) reservation: crate::branch::registry::ProductBranchRegistryReservation,
    pub(super) attempt: ReservedBranchCreationAttempt,
    pub(super) progress: CompositeAttemptProgress,
    pub(super) successor_basis: AdmittedCompositeRuntimeWorldBasis,
}

pub(super) fn install_forked_branch<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    installation: ForkedBranchInstallation,
    cancellation: &crate::publication::RuntimeWorldCancellationToken,
) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let finalization = state::ForkedBranchFinalization::from_installation(installation);
    let bound = match finalization.bind_publication() {
        Ok(bound) => bound,
        Err(effects) => {
            return Ok(RuntimeWorldBranchCreationOutcome::ProductUnpublished(
                effects,
            ))
        }
    };
    let observed = match bound.observe(&owner.state.retention, &owner.state.history) {
        Ok(observed) => observed,
        Err(effects) => {
            return Ok(RuntimeWorldBranchCreationOutcome::ProductUnpublished(
                effects,
            ))
        }
    };
    Ok(observed.install(cancellation))
}
