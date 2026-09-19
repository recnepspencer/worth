use std::sync::{Arc, Mutex, Weak};

use crate::branch::owner_services::{
    SignalOwner, SignalOwnerLifecycleIdentity, SignalOwnerUnavailable,
};
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial};
use crate::data::error::SignalError;
use crate::data::retained_storage::SignalConditionalRetentionReservation;
use crate::logic::transaction::TemporalRuntimeState;

use super::super::SignalConditionalServiceExecutionDenial;
use super::registry::SignalConditionalTemporalPartitionId;

/// A bounded temporal resource registered with its conditional service's owner.
/// The admitted basis remains pinned while later branch heads advance. This
/// capability does not authorize execution against a newer product occurrence.
pub struct SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) owner: Weak<SignalOwner<D, I, T>>,
    pub(super) cell: Weak<SignalConditionalTemporalCell>,
    pub(super) id: SignalConditionalTemporalPartitionId,
    pub(super) basis: AdmittedSignalBranchBasis,
    pub(super) owner_runtime_instance_id: u64,
    // Declared after `cell`: its weak allocation must drop before this custody.
    // Capacity stays conservatively reserved after owner close until the last
    // capability or in-flight cell drops. This never retains the Signal owner.
    pub(super) _custody: Arc<SignalConditionalRetentionReservation>,
}

#[derive(Debug)]
pub enum SignalConditionalTemporalPartitionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    ServiceAdmission(SignalConditionalServiceExecutionDenial),
    StaleBasisAdmission,
    StorageChargeOverflow,
    RetentionClosed,
    RetentionCapacityExhausted,
    RetentionQuarantined,
    ActiveWakeCapacityExhausted { maximum_active_wakes: usize },
    IdentityExhausted,
    PartitionRetired,
    PartitionQuarantined,
    RegistryQuarantined,
    PartitionAdmissionMismatch,
    OwnerStateAlreadyHeld,
    OwnerReentry,
    TemporalOperation(SignalError),
}

pub(super) struct SignalConditionalTemporalCell {
    pub(super) owner_runtime_instance_id: u64,
    pub(super) owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    pub(super) state: Mutex<SignalConditionalTemporalState>,
}

pub(super) struct SignalConditionalTemporalState {
    pub(super) temporal: TemporalRuntimeState,
    pub(super) maximum_active_wakes: usize,
    pub(super) quarantined: bool,
    #[cfg(test)]
    pub(super) promotion_fault: Option<super::promotion_fault::SignalTemporalPromotionFault>,
    pub(super) custody: Arc<SignalConditionalRetentionReservation>,
}

impl<D, I, T> SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn issuance_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.basis
    }

    /// Observational identity only; it grants no owner authority.
    pub fn owner_runtime_instance_id(&self) -> u64 {
        self.owner_runtime_instance_id
    }
}

impl<D, I, T> std::fmt::Debug for SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalConditionalTemporalPartition")
            .field("owner_runtime_instance_id", &self.owner_runtime_instance_id)
            .finish_non_exhaustive()
    }
}
