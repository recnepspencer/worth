#[cfg(test)]
use crate::branch::SignalBranchForkOperationDenial;
use std::sync::Arc;

use crate::branch::{
    SignalBranchAdvanceDenial, SignalBranchRestoreDenial, SignalBranchSnapshotCaptureDenial,
};
use crate::state::SignalBranchId;

use super::super::owner_metadata::{
    SignalOwnerMetadataAuthorizationDenial, SignalOwnerRetentionAcquisitionDenial,
};
use super::super::SignalOwnerOperationAdmission;
use super::super::{SignalBranchCellState, SignalBranchExecutionCell, SignalOwnerUnavailable};
use super::SignalOwner;
use crate::branch::retention::SignalBranchAdmissionReservation;

#[path = "output_retention/advance.rs"]
mod advance;

#[path = "output_retention/ready.rs"]
mod ready;

pub(in crate::branch::owner_services) struct SignalAdvanceOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: &'a SignalOwner<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    retention: SignalBranchAdmissionReservation,
}

pub(in crate::branch::owner_services) struct SignalSnapshotOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: &'a SignalOwner<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    branch_id: SignalBranchId,
    retention: SignalBranchAdmissionReservation,
}

pub(in crate::branch::owner_services) struct SignalRestoreOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: &'a SignalOwner<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    branch_id: SignalBranchId,
    retention: SignalBranchAdmissionReservation,
}

#[cfg(test)]
pub(in crate::branch::owner_services) struct SignalForkOutputReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: &'a SignalOwner<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    cell: Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    source_branch_id: SignalBranchId,
    retention: SignalBranchAdmissionReservation,
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_advance_output<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        cell: &Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    ) -> Result<SignalAdvanceOutputReservation<'a, D, I, T>, SignalBranchAdvanceDenial> {
        let branch_id = cell.branch_id();
        cell.validate_admission(admission).map_err(|denial| {
            super::super::branch_execution_cell::advance::map_advance_cell_denial(denial, branch_id)
        })?;
        let retention = self
            .reserve_output_retention(admission, branch_id, 1)
            .map_err(|denial| map_advance_output_denial(denial, branch_id))?;
        Ok(SignalAdvanceOutputReservation {
            owner: self,
            admission,
            cell: Arc::clone(cell),
            retention,
        })
    }

    pub(in crate::branch::owner_services) fn reserve_snapshot_outputs<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        cell: &Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    ) -> Result<SignalSnapshotOutputReservation<'a, D, I, T>, SignalBranchSnapshotCaptureDenial>
    {
        let branch_id = cell.branch_id();
        cell.validate_admission(admission).map_err(|denial| {
            super::super::branch_execution_cell::snapshot::map_snapshot_cell_denial(
                denial, branch_id,
            )
        })?;
        let retention = self
            .reserve_output_retention(admission, branch_id, 2)
            .map_err(|denial| map_snapshot_output_denial(denial, branch_id))?;
        Ok(SignalSnapshotOutputReservation {
            owner: self,
            admission,
            cell: Arc::clone(cell),
            branch_id,
            retention,
        })
    }

    pub(in crate::branch::owner_services) fn reserve_restore_output<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        cell: &Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    ) -> Result<SignalRestoreOutputReservation<'a, D, I, T>, SignalBranchRestoreDenial> {
        let branch_id = cell.branch_id();
        cell.validate_admission(admission).map_err(|denial| {
            super::super::branch_execution_cell::restoration::map_restore_cell_denial(
                denial, branch_id,
            )
        })?;
        let retention = self
            .reserve_output_retention(admission, branch_id, 1)
            .map_err(|denial| map_restore_output_denial(denial, branch_id))?;
        Ok(SignalRestoreOutputReservation {
            owner: self,
            admission,
            cell: Arc::clone(cell),
            branch_id,
            retention,
        })
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn reserve_fork_output<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        cell: &Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>,
    ) -> Result<SignalForkOutputReservation<'a, D, I, T>, SignalBranchForkOperationDenial> {
        let source_branch_id = cell.branch_id();
        cell.validate_admission(admission).map_err(|denial| {
            super::super::branch_execution_cell::fork::map_fork_cell_denial(
                denial,
                source_branch_id,
            )
        })?;
        let retention = self
            .reserve_output_retention(admission, source_branch_id, 1)
            .map_err(|denial| map_fork_output_denial(denial, source_branch_id))?;
        Ok(SignalForkOutputReservation {
            owner: self,
            admission,
            cell: Arc::clone(cell),
            source_branch_id,
            retention,
        })
    }

    fn reserve_output_retention(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        output_count: usize,
    ) -> Result<SignalBranchAdmissionReservation, SignalOutputRetentionDenial> {
        self.metadata.reserve_admitted_retention(
            admission,
            &self.retention,
            &self.counters,
            branch_id,
            output_count,
        )
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn reserve_admitted_output_slots_for_test(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        output_count: usize,
    ) -> Result<
        SignalBranchAdmissionReservation,
        crate::branch::SignalBranchRetentionAcquisitionDenial,
    > {
        self.reserve_admitted_retention(admission, branch_id, output_count)
    }
}

type SignalOutputRetentionDenial = SignalOwnerRetentionAcquisitionDenial;

macro_rules! map_output_denial {
    ($name:ident, $output:ty, $cell_misuse:expr, $retention:expr) => {
        fn $name(denial: SignalOutputRetentionDenial, branch_id: SignalBranchId) -> $output {
            match denial {
                SignalOutputRetentionDenial::Metadata(
                    SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable,
                ) => <$output>::OwnerUnavailable(SignalOwnerUnavailable),
                SignalOutputRetentionDenial::Metadata(
                    SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse,
                ) => $cell_misuse(branch_id),
                SignalOutputRetentionDenial::Metadata(
                    SignalOwnerMetadataAuthorizationDenial::OwnerReentry,
                ) => <$output>::OwnerReentry,
                SignalOutputRetentionDenial::Retention(denial) => $retention(denial),
            }
        }
    };
}

map_output_denial!(
    map_advance_output_denial,
    SignalBranchAdvanceDenial,
    |branch_id| SignalBranchAdvanceDenial::OwnerCellMisuse { branch_id },
    |denial| SignalBranchAdvanceDenial::RetentionUnavailable { denial }
);
map_output_denial!(
    map_snapshot_output_denial,
    SignalBranchSnapshotCaptureDenial,
    |branch_id| SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id },
    |denial| SignalBranchSnapshotCaptureDenial::RetentionUnavailable { denial }
);
map_output_denial!(
    map_restore_output_denial,
    SignalBranchRestoreDenial,
    |branch_id| SignalBranchRestoreDenial::OwnerCellMisuse { branch_id },
    |denial| SignalBranchRestoreDenial::RetentionUnavailable { denial }
);
#[cfg(test)]
map_output_denial!(
    map_fork_output_denial,
    SignalBranchForkOperationDenial,
    |branch_id| SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id },
    |denial| SignalBranchForkOperationDenial::RetentionUnavailable { denial }
);
