use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchForkOperationDenial, SignalBranchObservation,
    ValidatedSignalBranchName,
};
use crate::data::error::SignalError;
use crate::logic::transaction::BranchState;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::branch_execution_cell::SignalBranchForkSourceCustody;
use super::super::branch_registry::{
    SignalBranchReservation, SignalPreparedBranchCell, SignalPreparedBranchInstallation,
};
use super::super::operation_control::SignalOwnerOperationBoundary;
use super::super::owner_metadata::SignalOwnerForkLineageReservation;
use super::super::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalOwnerCancellationToken,
    SignalOwnerOperationAdmission, SignalOwnerUnavailable,
};
use super::fork_denials::{map_fork_owner_lock_denial, map_fork_registry_denial};

impl<D, I, T> super::SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn issue_fork_destination_identity(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        source: &AdmittedSignalBranchBasis,
        requested_identity: ValidatedSignalBranchName,
    ) -> Result<SignalBranchHandle, SignalBranchForkOperationDenial> {
        let parent_branch_id = source.owner_branch_id();
        admission
            .authorize(self.runtime_instance_id, self.lifecycle_identity())
            .map_err(|_| {
                SignalBranchForkOperationDenial::OwnerUnavailable(SignalOwnerUnavailable)
            })?;
        admission
            .owner_lock_acquisition()
            .map_err(|denial| map_fork_owner_lock_denial(denial, parent_branch_id))?;
        let branch_id = self
            .next_branch_id
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map(SignalBranchId)
            .map_err(|_| SignalBranchForkOperationDenial::BranchIdentityExhausted)?;
        let parent_head_snapshot_id = source
            .observation()
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id())
            .map(SignalSnapshotId);
        Ok(SignalBranchHandle {
            id: branch_id,
            name: requested_identity.into_inner(),
            parent_branch_id: Some(parent_branch_id),
            head_snapshot_id: parent_head_snapshot_id,
        })
    }

    pub(in crate::branch::owner_services) fn reserve_fork_destination<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        source: &AdmittedSignalBranchBasis,
        requested_identity: ValidatedSignalBranchName,
    ) -> Result<SignalOwnerForkReservation<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let handle = self.issue_fork_destination_identity(admission, source, requested_identity)?;
        let branch_id = handle.id;
        let reservation = self
            .registry
            .reserve_named(admission, branch_id, handle.name.clone())
            .map_err(|denial| map_fork_registry_denial(denial, source.owner_branch_id()))?;
        let lineage =
            self.metadata
                .reserve_fork_child(admission, source.owner_branch_id(), branch_id)?;
        Ok(SignalOwnerForkReservation::new(
            handle,
            self.runtime_instance_id,
            self.definition_basis,
            reservation,
            lineage,
        ))
    }
}

pub(in crate::branch::owner_services) struct SignalOwnerForkReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    handle: SignalBranchHandle,
    owner_runtime_instance_id: u64,
    definition_basis: u64,
    reservation: SignalBranchReservation<'a, SignalBranchCellState<D, I, T>>,
    lineage: SignalOwnerForkLineageReservation<'a, D, I, T>,
}

impl<'a, D, I, T> SignalOwnerForkReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn new(
        handle: SignalBranchHandle,
        owner_runtime_instance_id: u64,
        definition_basis: u64,
        reservation: SignalBranchReservation<'a, SignalBranchCellState<D, I, T>>,
        lineage: SignalOwnerForkLineageReservation<'a, D, I, T>,
    ) -> Self {
        Self {
            handle,
            owner_runtime_instance_id,
            definition_basis,
            reservation,
            lineage,
        }
    }

    pub(in crate::branch::owner_services) fn branch(&self) -> &SignalBranchHandle {
        &self.handle
    }

    pub(in crate::branch::owner_services) fn capture(
        self,
        source_custody: SignalBranchForkSourceCustody<'a, 'a, SignalBranchCellState<D, I, T>>,
        source: &AdmittedSignalBranchBasis,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalPreparedOwnerFork<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let Some(parent_branch_id) = self.handle.parent_branch_id else {
            return Err(owner_fork_invariant(
                "fork destination reservation has no source branch",
            ));
        };
        if source_custody.cell().branch_id() != parent_branch_id
            || source.owner_branch_id() != parent_branch_id
        {
            return Err(owner_fork_invariant(
                "fork source does not match its owner-issued destination reservation",
            ));
        }
        let admission = self.reservation.admission();
        let source_cell = source_custody.cell_arc();
        let builder = SignalOwnerForkCellBuilder {
            source_custody: Some(source_custody),
            handle: self.handle,
            owner_runtime_instance_id: self.owner_runtime_instance_id,
            definition_basis: self.definition_basis,
            reservation: self.reservation,
            lineage: self.lineage,
            destination_observation: None,
        };
        source_cell.capture_fork_source_exact(admission, source, builder, cancellation)
    }

    pub(in crate::branch::owner_services) fn capture_and_install(
        self,
        source_cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
        source: &AdmittedSignalBranchBasis,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalInstalledOwnerFork<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let source_branch_id = source.owner_branch_id();
        let source_custody = source_cell
            .acquire_fork_source_custody(self.reservation.admission())
            .map_err(|denial| {
                crate::branch::owner_services::branch_execution_cell::fork::map_fork_cell_denial(
                    denial,
                    source_branch_id,
                )
            })?;
        self.capture(source_custody, source, cancellation)?
            .install()
    }
}

pub(crate) struct SignalOwnerForkCellBuilder<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    source_custody: Option<SignalBranchForkSourceCustody<'a, 'a, SignalBranchCellState<D, I, T>>>,
    handle: SignalBranchHandle,
    owner_runtime_instance_id: u64,
    definition_basis: u64,
    reservation: SignalBranchReservation<'a, SignalBranchCellState<D, I, T>>,
    lineage: SignalOwnerForkLineageReservation<'a, D, I, T>,
    destination_observation: Option<SignalBranchObservation>,
}

pub(crate) struct SignalPreparedOwnerFork<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    handle: SignalBranchHandle,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    installation: SignalPreparedBranchInstallation<'a, SignalBranchCellState<D, I, T>>,
    lineage: SignalOwnerForkLineageReservation<'a, D, I, T>,
    destination_observation: SignalBranchObservation,
}

pub(in crate::branch::owner_services) struct SignalInstalledOwnerFork<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    handle: SignalBranchHandle,
    observation: SignalBranchObservation,
    cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    lifetime: std::marker::PhantomData<&'a SignalOwnerOperationAdmission<'a>>,
}

impl<'a, D, I, T> SignalInstalledOwnerFork<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn cell(
        &self,
    ) -> &Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>> {
        &self.cell
    }

    pub(in crate::branch::owner_services) fn into_handoff_parts(
        self,
    ) -> (SignalBranchHandle, SignalBranchObservation) {
        (self.handle, self.observation)
    }
}

impl<D, I, T> std::ops::Deref for SignalInstalledOwnerFork<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    type Target = SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl<'a, D, I, T> SignalOwnerForkCellBuilder<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn source_matches(
        &self,
        cell: &SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>,
    ) -> bool {
        self.source_custody
            .as_ref()
            .is_some_and(|custody| custody.matches(cell))
    }

    pub(crate) fn destination(&self) -> &SignalBranchHandle {
        &self.handle
    }

    pub(crate) fn prepare_cell(
        &mut self,
        state: BranchState<D, I, T>,
    ) -> Result<
        SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
        SignalBranchForkOperationDenial,
    > {
        if state.branch_id() != self.handle.id {
            return Err(owner_fork_invariant(
                "fork destination state does not match its owner-issued reservation",
            ));
        }
        let destination_branch_id = self.handle.id;
        let destination_state = SignalBranchCellState::new(
            self.handle.clone(),
            self.owner_runtime_instance_id,
            self.definition_basis,
            state,
            0,
            None,
        );
        let destination_observation = destination_state
            .observation()
            .map_err(|error| SignalBranchForkOperationDenial::OwnerDeniedNoMovement { error })?;
        let prepared = self
            .reservation
            .prepare_fork_destination_cell(destination_state)
            .map_err(|denial| map_fork_registry_denial(denial, destination_branch_id))?;
        self.destination_observation = Some(destination_observation);
        Ok(prepared)
    }

    pub(crate) fn validate_prepared_cell(
        &self,
        prepared: &SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
    ) -> Result<(), SignalBranchForkOperationDenial> {
        self.reservation
            .matches_prepared_fork_destination(prepared)
            .then_some(())
            .ok_or_else(|| {
                owner_fork_invariant(
                    "prepared fork destination does not match its exact owner reservation",
                )
            })
    }

    pub(crate) fn bind_prepared_cell(
        mut self,
        prepared: SignalPreparedBranchCell<SignalBranchCellState<D, I, T>>,
    ) -> SignalPreparedOwnerFork<'a, D, I, T> {
        drop(
            self.source_custody
                .take()
                .expect("prepared fork capture retains source custody until capture completes"),
        );
        let admission = self.reservation.admission();
        SignalPreparedOwnerFork {
            handle: self.handle,
            admission,
            installation: self.reservation.bind_prepared_fork_destination(prepared),
            lineage: self.lineage,
            destination_observation: self
                .destination_observation
                .expect("prepared fork destination must carry its sealed observation"),
        }
    }
}

impl<'a, D, I, T> SignalPreparedOwnerFork<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn install(
        self,
    ) -> Result<SignalInstalledOwnerFork<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let SignalPreparedOwnerFork {
            handle,
            admission,
            installation,
            lineage,
            destination_observation,
        } = self;
        let branch_id = handle.id;
        let installation = installation
            .install_recoverable()
            .map_err(|denial| map_fork_registry_denial(denial, branch_id))?;
        let cell = Arc::clone(installation.cell());
        lineage.commit();
        installation.commit();
        admission
            .reach_operation_boundary(SignalOwnerOperationBoundary::ForkDestinationInstallation);
        Ok(SignalInstalledOwnerFork {
            handle,
            observation: destination_observation,
            cell,
            lifetime: std::marker::PhantomData,
        })
    }
}

fn owner_fork_invariant(message: &str) -> SignalBranchForkOperationDenial {
    SignalBranchForkOperationDenial::OwnerDeniedNoMovement {
        error: SignalError::internal(message),
    }
}
