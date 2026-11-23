//! Historical root walk verification, delta check, and fingerprint assembly.
use super::super::*;
use super::final_pass::FinalPassObservation;
use crate::physical_runtime::SharedRecoveryCheckpoint;

#[inline(never)]
pub(super) fn verify_and_finalize(
    coordination: &PhysicalRecoveryCoordination,
    final_obs: FinalPassObservation<'_>,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedPendingWalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    checkpoint: &SharedRecoveryCheckpoint,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedWalMediaFingerprint,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    // Bind in original acquisition order so refusal/unwind drops remaining
    // owners in reverse order: final_inventory, final_heads, final_controls,
    // final_selected, media, projection, sample.
    let FinalPassObservation {
        sample,
        projection,
        media,
        final_selected,
        final_controls,
        final_heads,
        final_inventory,
    } = final_obs;
    let (media, historical_fingerprint) = if claim.ordered_history().is_some() {
        if !claim.historical_batches().is_empty() {
            return Err(Denial::CertificateRoster);
        }
        let retained = resident_budget::retained_with_heads(
            &final_selected,
            &final_controls,
            claim,
            &projection,
            &sample,
            checkpoint,
            &final_heads,
        )?;
        let mut read_allocation =
            crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(coordination)
                .map_err(Denial::WalReadOwnership)?;
        verify_ordered_history(
            media,
            &mut read_allocation,
            reopen,
            claim,
            effective,
            &final_controls,
            &final_inventory,
            &sample,
            retained,
        )?
    } else {
        match claim.historical_batches() {
            [] => (media, None),
            [batch] => {
                let retained = delta::retained_memory(
                    &final_selected,
                    &final_controls,
                    claim,
                    &projection,
                    &sample,
                    checkpoint,
                )?;
                verify_historical_batch(
                    media,
                    reopen,
                    claim,
                    batch,
                    &final_inventory,
                    &sample,
                    retained,
                )?
            }
            _ => return Err(Denial::CertificateRoster),
        }
    };
    let wal_fingerprint = final_inventory.into_fingerprint();
    let media = delta::verify(
        media,
        reopen,
        &final_selected,
        &final_controls,
        &final_heads,
        claim,
        &projection,
        &sample,
        checkpoint,
    )?;
    let mut controls_fingerprint = final_controls.into_fingerprint()?;
    controls_fingerprint.extend(final_heads.into_fingerprint()?)?;
    if let Some((historical, retained)) = historical_fingerprint {
        resident_budget::extend_historical(&mut controls_fingerprint, historical, retained)?;
    }
    Ok((media, wal_fingerprint, controls_fingerprint))
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn verify_ordered_history(
    media: AdmittedRecoveryFilesystemMedia,
    read_allocation: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedPendingWalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    final_controls: &controls::Controls,
    final_inventory: &wal_inventory::AdmittedWalInventory,
    sample: &crate::physical_runtime::StoreRecoveryBindingFreshnessSample,
    retained: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        Option<(SelectedControlMediaFingerprint, u64)>,
    ),
    Denial,
> {
    let (media, fingerprint) = ordered_walk::verify(
        media,
        read_allocation,
        claim,
        effective,
        final_controls,
        sample,
        final_inventory.frames(),
        reopen.format(),
        retained,
    )?;
    Ok((media, Some((fingerprint, retained))))
}

#[allow(clippy::too_many_arguments)]
#[inline(never)]
fn verify_historical_batch(
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedPendingWalReleaseCustody,
    batch: &worth_store_recovery_physics::VerifiedHistoricalPendingWalBatch,
    final_inventory: &wal_inventory::AdmittedWalInventory,
    sample: &crate::physical_runtime::StoreRecoveryBindingFreshnessSample,
    retained: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        Option<(SelectedControlMediaFingerprint, u64)>,
    ),
    Denial,
> {
    let (media, mut history_fingerprint) = historical_chain::verify_ordinary_parts(
        media,
        batch.chain(),
        claim.checkpoint().source().root().generation(),
        claim.source_root().generation(),
        claim.source_root_sha256(),
        sample,
        final_inventory.frames(),
        claim
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
        claim.descriptor().custody().request().idempotency(),
        claim.wal_fate().lsn_start(),
        reopen.format(),
        claim.source_root().node_capacity(),
        retained,
    )?;
    let (media, fingerprint) = historical_first::verify(
        media,
        batch,
        sample,
        final_inventory.frames(),
        claim
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
        reopen.format(),
        claim.source_root().node_capacity(),
        retained,
    )?;
    history_fingerprint.try_extend_bounded(
        fingerprint,
        delta::MAX_TRANSITION_MEMORY
            .checked_sub(retained)
            .ok_or(Denial::BoundExceeded)?,
    )?;
    Ok((media, Some((history_fingerprint, retained))))
}
