use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::lifecycle_state::SignalOwnerBranchCellHoldDenial;
use crate::branch::owner_services::SignalOwner;

use super::partition::{SignalConditionalTemporalCell, SignalConditionalTemporalState};
use super::{
    SignalConditionalTemporalPartition, SignalConditionalTemporalPartitionDenial as Denial,
};

impl<D, I, T> SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Dispose this registered resource through its owner. The capability is consumed.
    pub fn retire(self) -> Result<(), Denial> {
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let _admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        owner.conditional_temporal.retire(self.id);
        Ok(())
    }

    /// Admission and locking touch only the carried owner lifecycle and partition.
    pub(super) fn with_admitted_partition<R>(
        &self,
        operation: impl FnOnce(&mut SignalConditionalTemporalState) -> Result<R, Denial>,
    ) -> Result<R, Denial> {
        self.with_partition_cell(|cell| {
            let mut state = cell
                .state
                .lock()
                .map_err(|_| Denial::PartitionQuarantined)?;
            if state.quarantined {
                return Err(Denial::PartitionQuarantined);
            }
            operation(&mut state)
        })
    }

    /// Cleanup diagnostics can count retained records even after a failed effect.
    /// Recovering this lock grants no mutable access and never clears quarantine.
    pub(super) fn inspect_retained_wakes(
        &self,
    ) -> Result<crate::data::temporal::TemporalWakeSummary, Denial> {
        self.with_partition_cell(|cell| {
            let state = cell
                .state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            Ok(state.temporal.wake_summary())
        })
    }

    fn with_partition_cell<R>(
        &self,
        operation: impl FnOnce(&SignalConditionalTemporalCell) -> Result<R, Denial>,
    ) -> Result<R, Denial> {
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let cell = self.cell.upgrade().ok_or(Denial::PartitionRetired)?;
        admission
            .authorize(
                cell.owner_runtime_instance_id,
                cell.owner_lifecycle_identity,
            )
            .map_err(|_| Denial::PartitionAdmissionMismatch)?;
        let _hold = admission
            .hold_branch_cell()
            .map_err(|denial| match denial {
                SignalOwnerBranchCellHoldDenial::SecondCellWhileHeld => {
                    Denial::OwnerStateAlreadyHeld
                }
                SignalOwnerBranchCellHoldDenial::ExecutingThreadReentry => Denial::OwnerReentry,
            })?;
        operation(&cell)
    }
}

impl<D, I, T> Drop for SignalConditionalTemporalPartition<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            owner.conditional_temporal.retire(self.id);
        }
    }
}
