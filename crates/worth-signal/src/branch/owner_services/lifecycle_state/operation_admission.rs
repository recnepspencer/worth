use std::fmt;
use std::sync::Arc;

use super::super::admission_table::{
    SignalOwnerAdmissionHold, SignalOwnerAdmissionHoldDenial, SignalOwnerPublishedAdmission,
};
use super::{
    SignalOwnerAdmissionMismatch, SignalOwnerCloseCoordinator, SignalOwnerLifecycleIdentity,
    SignalOwnerLifecycleState,
};
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

pub(crate) struct SignalOwnerOperationAdmission<'owner> {
    lifecycle: Arc<SignalOwnerLifecycleState>,
    owner_runtime_instance_id: u64,
    lifecycle_identity: SignalOwnerLifecycleIdentity,
    published: SignalOwnerPublishedAdmission,
    close_coordinator: Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>>,
}

impl fmt::Debug for SignalOwnerOperationAdmission<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignalOwnerOperationAdmission")
            .field("owner_runtime_instance_id", &self.owner_runtime_instance_id)
            .finish_non_exhaustive()
    }
}

impl<'owner> SignalOwnerOperationAdmission<'owner> {
    pub(super) fn new(
        lifecycle: Arc<SignalOwnerLifecycleState>,
        owner_runtime_instance_id: u64,
        lifecycle_identity: SignalOwnerLifecycleIdentity,
        published: SignalOwnerPublishedAdmission,
        close_coordinator: Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>>,
    ) -> Self {
        Self {
            lifecycle,
            owner_runtime_instance_id,
            lifecycle_identity,
            published,
            close_coordinator,
        }
    }

    pub(in crate::branch::owner_services) fn authorize(
        &self,
        owner_runtime_instance_id: u64,
        lifecycle_identity: SignalOwnerLifecycleIdentity,
    ) -> Result<(), SignalOwnerAdmissionMismatch> {
        if self.owner_runtime_instance_id != owner_runtime_instance_id {
            return Err(SignalOwnerAdmissionMismatch::ForeignOwner);
        }
        if self.lifecycle_identity != lifecycle_identity {
            return Err(SignalOwnerAdmissionMismatch::ExpiredLifecycle);
        }
        Ok(())
    }

    pub(in crate::branch::owner_services) fn hold_branch_cell(
        &self,
    ) -> Result<SignalOwnerBranchCellHold<'_>, SignalOwnerBranchCellHoldDenial> {
        self.reach_operation_boundary(SignalOwnerOperationBoundary::TargetCellAdmission);
        let (hold, scanned) = self.published.hold_cell().map_err(|(denial, scanned)| {
            self.lifecycle
                .counters
                .record_admission_records_scanned(scanned);
            match denial {
                SignalOwnerAdmissionHoldDenial::AdmissionAlreadyHoldsOwnerState => {
                    SignalOwnerBranchCellHoldDenial::SecondCellWhileHeld
                }
                SignalOwnerAdmissionHoldDenial::ExecutingThreadReentry => {
                    SignalOwnerBranchCellHoldDenial::ExecutingThreadReentry
                }
            }
        })?;
        self.lifecycle
            .counters
            .record_admission_records_scanned(scanned);
        Ok(SignalOwnerBranchCellHold { _hold: hold })
    }

    pub(in crate::branch::owner_services) fn reach_operation_boundary(
        &self,
        boundary: SignalOwnerOperationBoundary,
    ) {
        self.lifecycle.reach_operation_boundary(boundary);
    }

    pub(in crate::branch::owner_services) fn hold_owner_metadata(
        &self,
    ) -> Result<SignalOwnerMetadataHold<'_>, SignalOwnerMetadataHoldDenial> {
        let (hold, scanned) = self
            .published
            .hold_metadata()
            .map_err(|(denial, scanned)| {
                self.lifecycle
                    .counters
                    .record_admission_records_scanned(scanned);
                match denial {
                    SignalOwnerAdmissionHoldDenial::AdmissionAlreadyHoldsOwnerState => {
                        SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld
                    }
                    SignalOwnerAdmissionHoldDenial::ExecutingThreadReentry => {
                        SignalOwnerMetadataHoldDenial::ExecutingThreadReentry
                    }
                }
            })?;
        self.lifecycle
            .counters
            .record_admission_records_scanned(scanned);
        Ok(SignalOwnerMetadataHold { _hold: hold })
    }

    pub(in crate::branch::owner_services) fn permits_owner_lock_acquisition(&self) -> bool {
        self.owner_lock_acquisition().is_ok()
    }

    pub(in crate::branch::owner_services) fn owner_lock_acquisition(
        &self,
    ) -> Result<(), SignalOwnerMetadataHoldDenial> {
        match self.published.can_acquire_owner_lock() {
            Ok(((), scanned)) => {
                self.lifecycle
                    .counters
                    .record_admission_records_scanned(scanned);
                Ok(())
            }
            Err((denial, scanned)) => {
                self.lifecycle
                    .counters
                    .record_admission_records_scanned(scanned);
                Err(match denial {
                    SignalOwnerAdmissionHoldDenial::AdmissionAlreadyHoldsOwnerState => {
                        SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld
                    }
                    SignalOwnerAdmissionHoldDenial::ExecutingThreadReentry => {
                        SignalOwnerMetadataHoldDenial::ExecutingThreadReentry
                    }
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch::owner_services) enum SignalOwnerBranchCellHoldDenial {
    SecondCellWhileHeld,
    ExecutingThreadReentry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch::owner_services) enum SignalOwnerMetadataHoldDenial {
    BranchCellOrMetadataAlreadyHeld,
    ExecutingThreadReentry,
}

pub(in crate::branch::owner_services) struct SignalOwnerBranchCellHold<'a> {
    _hold: SignalOwnerAdmissionHold<'a>,
}

pub(in crate::branch::owner_services) struct SignalOwnerMetadataHold<'a> {
    _hold: SignalOwnerAdmissionHold<'a>,
}

impl Drop for SignalOwnerOperationAdmission<'_> {
    fn drop(&mut self) {
        debug_assert!(self.published.is_idle());
        let scanned = self.published.unpublish();
        self.lifecycle
            .counters
            .record_admission_records_scanned(scanned);
        if self.lifecycle.release_operation() {
            if let Some(coordinator) = self.close_coordinator.take() {
                coordinator.finish_owner_close();
            } else if let Some(claim) = self.lifecycle.claim_cleanup() {
                claim.complete();
            }
        }
    }
}

pub(super) struct SignalOwnerPendingAdmission<'owner> {
    pub(super) lifecycle: Arc<SignalOwnerLifecycleState>,
    pub(super) close_coordinator: Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>>,
    pub(super) committed: bool,
}

impl<'owner> SignalOwnerPendingAdmission<'owner> {
    pub(super) fn commit(mut self) -> Option<Arc<dyn SignalOwnerCloseCoordinator + 'owner>> {
        self.committed = true;
        self.close_coordinator.take()
    }
}

impl Drop for SignalOwnerPendingAdmission<'_> {
    fn drop(&mut self) {
        if self.committed || !self.lifecycle.release_operation() {
            return;
        }
        if let Some(coordinator) = self.close_coordinator.take() {
            coordinator.finish_owner_close();
        } else if let Some(claim) = self.lifecycle.claim_cleanup() {
            claim.complete();
        }
    }
}
