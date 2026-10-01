//! Current-media verification and allocation-first recovered-ledger lowering.
//! This phase completes before Core progression or Serving owner installation.

use crate::physical_runtime::{
    durability::PreparedRecoveredCheckpointCustody,
    record_serving::{RecordBootstrapDenial, RecordServingState},
    MediaOwnedPhysicalRuntime, RecoveredPhysicalCheckpointCustody,
};
use worth_store_buffer_pool::OperationAllocationGrant;

pub(super) fn prepare(
    custody: Option<RecoveredPhysicalCheckpointCustody>,
    runtime: &MediaOwnedPhysicalRuntime,
    state: &RecordServingState,
    grant: &OperationAllocationGrant,
) -> Result<Option<PreparedRecoveredCheckpointCustody>, RecordBootstrapDenial> {
    let Some(custody) = custody else {
        return if state.current_root.tier_epoch_anchor().is_some()
            || state.current_root.release_custody_head_root().is_some()
        {
            Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)
        } else {
            Ok(None)
        };
    };
    let verified = custody
        .verify_for_serving(
            runtime.record_serving_media(),
            runtime.store_identity(),
            &state.current_root,
            &state.free_space,
            state.format.declaration(),
        )
        .map_err(|_| RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
    #[cfg(feature = "recovery-runtime-owner")]
    {
        use crate::physical_runtime::{
            durability::RecoveredReleaseLedgerDenial,
            recovery_residency::StoreRejoinResidentLedger,
            PhysicalRecoveryRejoinResidentDenial as ResidentDenial,
        };
        let allocation = verified.seal().recovery_allocation_admission();
        let already_live = verified
            .seal()
            .owned_heap_bytes()
            .and_then(|seal| {
                state
                    .owned_heap_bytes()
                    .and_then(|bootstrap| seal.checked_add(bootstrap))
            })
            .ok_or(RecordBootstrapDenial::RecoveredCustodyResident(
                ResidentDenial::SizeOverflow {
                    admitted: allocation.byte_limit(),
                },
            ))?;
        let mut resident = StoreRejoinResidentLedger::from_retained_with_limit(
            allocation,
            already_live,
            grant.bytes(),
        )
        .map_err(RecordBootstrapDenial::RecoveredCustodyResident)?;
        PreparedRecoveredCheckpointCustody::prepare(verified, &mut resident)
            .map(Some)
            .map_err(|denial| match denial {
                RecoveredReleaseLedgerDenial::Resident(cause) => {
                    RecordBootstrapDenial::RecoveredCustodyResident(cause)
                }
                RecoveredReleaseLedgerDenial::SelectedFactMismatch => {
                    RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch
                }
            })
    }
    #[cfg(not(feature = "recovery-runtime-owner"))]
    {
        let _ = (verified, grant);
        Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)
    }
}
