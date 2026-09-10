use std::sync::Arc;

use super::denials::{map_fork_admission_denial, map_fork_registry_denial};
use super::{SignalBranchMutationPort, SignalOwnerCancellationToken};
use crate::branch::owner_services::owner::fork_destination::SignalOwnedForkDestination;
use crate::branch::owner_services::owner::fork_reservation::SignalOwnerForkReservation;
use crate::branch::owner_services::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwner, SignalOwnerLifecycleObservation,
    SignalOwnerUnavailable,
};
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchForkOperationDenial, SignalBranchForkOutcome,
};

/// Move-only owner-issued destination reservation for an exact Signal fork.
///
/// The reservation retains the issuing owner state, source basis and cell,
/// destination name and identity, registry/name occupancy, fork lineage, and
/// one real admitted-output retention slot. Dropping it releases every still
/// pending component through its owning authority.
#[must_use = "dropping the reservation releases its fork capacity"]
pub struct SignalBranchForkReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Arc<SignalOwner<D, I, T>>,
    source: AdmittedSignalBranchBasis,
    source_cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    destination: SignalOwnedForkDestination<D, I, T>,
}

impl<D, I, T> std::fmt::Debug for SignalBranchForkReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalBranchForkReservation")
            .field(
                "owner_runtime_instance_id",
                &self.owner.runtime_instance_id(),
            )
            .field("source_branch_id", &self.source.owner_branch_id())
            .field("destination_branch_id", &self.destination.handle.id)
            .field("destination_name", &self.destination.handle.name)
            .finish_non_exhaustive()
    }
}

impl<D, I, T> SignalBranchForkReservation<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn issue<E, Ctx>(
        port: &SignalBranchMutationPort<D, I, E, Ctx, T>,
        requested_identity: crate::branch::ValidatedSignalBranchName,
        source: &AdmittedSignalBranchBasis,
    ) -> Result<Self, SignalBranchForkOperationDenial> {
        Self::issue_with_source_preflight(port, requested_identity, source, true)
    }

    pub(super) fn issue_for_immediate_fork<E, Ctx>(
        port: &SignalBranchMutationPort<D, I, E, Ctx, T>,
        requested_identity: crate::branch::ValidatedSignalBranchName,
        source: &AdmittedSignalBranchBasis,
    ) -> Result<Self, SignalBranchForkOperationDenial> {
        Self::issue_with_source_preflight(port, requested_identity, source, false)
    }

    fn issue_with_source_preflight<E, Ctx>(
        port: &SignalBranchMutationPort<D, I, E, Ctx, T>,
        requested_identity: crate::branch::ValidatedSignalBranchName,
        source: &AdmittedSignalBranchBasis,
        preflight_source: bool,
    ) -> Result<Self, SignalBranchForkOperationDenial> {
        let owner = port
            .upgrade_owner()
            .map_err(SignalBranchForkOperationDenial::OwnerUnavailable)?;
        let admission_owner = Arc::clone(&owner);
        let admission = admission_owner.admit().map_err(map_fork_admission_denial)?;
        let source_branch_id = source.owner_branch_id();
        let source_cell = owner
            .lookup_cell(&admission, source_branch_id)
            .map_err(|denial| map_fork_registry_denial(denial, source_branch_id))?;
        if preflight_source {
            source_cell.preflight_fork_source_exact(&admission, source)?;
        }
        let destination =
            owner.reserve_fork_destination_owned(&admission, source, requested_identity)?;
        Ok(Self {
            owner,
            source: source.clone(),
            source_cell,
            destination,
        })
    }

    pub(super) fn consume_for<E, Ctx>(
        self,
        port: &SignalBranchMutationPort<D, I, E, Ctx, T>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchForkOutcome, SignalBranchForkOperationDenial> {
        if !port.matches_owner(&self.owner) {
            return Err(SignalBranchForkOperationDenial::OwnerUnavailable(
                SignalOwnerUnavailable,
            ));
        }
        if self.owner.lifecycle_observation() != SignalOwnerLifecycleObservation::Open {
            return Err(SignalBranchForkOperationDenial::OwnerUnavailable(
                SignalOwnerUnavailable,
            ));
        }
        let Self {
            owner,
            source,
            source_cell,
            destination,
        } = self;
        let SignalOwnedForkDestination {
            handle,
            owner_runtime_instance_id,
            definition_basis,
            registry,
            lineage,
            mut retention,
        } = destination;
        let admission_owner = Arc::clone(&owner);
        let admission = admission_owner.admit().map_err(map_fork_admission_denial)?;
        let registry = registry.into_borrowed(owner.registry(), &admission);
        let lineage = lineage.into_borrowed(&owner.metadata, &admission);
        let destination = SignalOwnerForkReservation::new(
            handle,
            owner_runtime_instance_id,
            definition_basis,
            registry,
            lineage,
        );
        let installed = destination.capture_and_install(source_cell, &source, cancellation)?;
        admission.reach_operation_boundary(
            crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary::OutcomeConstruction,
        );
        let destination_cell_incarnation = installed.cell().incarnation().get();
        let (created_branch, observation) = installed.into_handoff_parts();
        let created_basis = owner.admit_canonical_basis(
            observation,
            created_branch.id,
            destination_cell_incarnation,
            retention.take_one(),
        );
        let retirement_reference = owner
            .issue_managed_branch_reference_with_admission(&admission, &created_basis)
            .expect("a just-created branch remains admitted in the same owner operation");
        Ok(SignalBranchForkOutcome::owner_issued(
            created_branch,
            created_basis,
            retirement_reference,
        ))
    }
}
