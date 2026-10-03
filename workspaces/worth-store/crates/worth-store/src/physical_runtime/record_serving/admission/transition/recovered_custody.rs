//! Current-media verification and allocation-first recovered-ledger lowering.
//! This phase completes before Core progression or Serving owner installation.

use crate::physical_runtime::{
    durability::{CheckpointCustodyCandidate, PreparedRecoveredCheckpointCustody},
    instance::PhysicalResidencyOwner,
    record_serving::{RecordBootstrapDenial, RecordServingState},
    MediaOwnedPhysicalRuntime,
};
use worth_store_buffer_pool::OperationAllocationGrant;

pub(super) fn prepare(
    custody: Option<super::super::recovered_custody::RecoveredCheckpointCustodyEvidence>,
    clean: CheckpointCustodyCandidate,
    runtime: &MediaOwnedPhysicalRuntime,
    state: &RecordServingState,
    residency: &PhysicalResidencyOwner,
    grant: &mut OperationAllocationGrant,
) -> Result<Option<PreparedRecoveredCheckpointCustody>, RecordBootstrapDenial> {
    let Some(custody) = custody else {
        // A tier-anchored root opens without C.8 only through clean tier
        // custody; a release custody head always requires C.8.
        return if (state.current_root.tier_epoch_anchor().is_some()
            && !clean.requires_clean_custody())
            || state.current_root.release_custody_head_root().is_some()
        {
            Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)
        } else {
            Ok(None)
        };
    };
    #[cfg(feature = "recovery-runtime-owner")]
    {
        let funded_media_verified = verify_funded_media(custody, runtime, state, residency, grant)?;
        let verified = funded_media_verified
            .verify_for_serving(
                runtime.record_serving_media(),
                runtime.store_identity(),
                &state.current_root,
                &state.free_space,
                state.format.declaration(),
            )
            .map_err(|_| RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
        use crate::physical_runtime::{
            durability::RecoveredReleaseLedgerDenial,
            recovery_residency::StoreRejoinResidentLedger,
            PhysicalRecoveryRejoinResidentDenial as ResidentDenial,
        };
        let allocation = verified.seal().recovery_allocation_admission();
        let already_live = verified
            .seal()
            .bootstrap_covered_heap_bytes()
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
        let _ = (custody, runtime, state, residency, grant);
        Err(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)
    }
}

#[cfg(feature = "recovery-runtime-owner")]
fn verify_funded_media(
    custody: super::super::recovered_custody::RecoveredCheckpointCustodyEvidence,
    runtime: &MediaOwnedPhysicalRuntime,
    state: &RecordServingState,
    residency: &PhysicalResidencyOwner,
    grant: &mut OperationAllocationGrant,
) -> Result<
    super::super::recovered_custody::FundedMediaVerifiedRecoveredCustodyEvidence,
    RecordBootstrapDenial,
> {
    use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial as ResidentDenial;
    let preparation_bytes = grant.bytes();
    let retained =
        state
            .owned_heap_bytes()
            .ok_or(RecordBootstrapDenial::RecoveredCustodyResident(
                ResidentDenial::SizeOverflow {
                    admitted: preparation_bytes,
                },
            ))?;
    if retained > preparation_bytes {
        return Err(RecordBootstrapDenial::RecoveredCustodyResident(
            ResidentDenial::BudgetExceeded {
                required: retained,
                admitted: preparation_bytes,
            },
        ));
    }
    // record_open/load_current_root returned; their transient buffers are gone.
    // The actual state vectors stay backed throughout this read and restoration.
    grant
        .try_resize(retained)
        .map_err(RecordBootstrapDenial::from_residency)?;
    let mut window = crate::physical_runtime::PhysicalRecoveryReadAllocation::for_serving(
        residency,
        runtime.lifecycle_state().snapshot().generation,
    );
    let verified = custody.verify_wal_for_serving(
        runtime.record_serving_media(),
        runtime.store_identity(),
        &state.current_root,
        &mut window,
    )?;
    let verified =
        verified.verify_selected_media_for_serving(runtime.record_serving_media(), &mut window)?;
    drop(window);
    // Real native growth precedes remaining control checks and ledger lowering;
    // a competing owner can deny restoration, with its exact native cause.
    grant
        .try_resize(preparation_bytes)
        .map_err(RecordBootstrapDenial::from_residency)?;
    Ok(verified)
}
