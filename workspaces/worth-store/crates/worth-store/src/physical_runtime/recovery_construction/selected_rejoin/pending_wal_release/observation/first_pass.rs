//! First-pass selected-media observation and WAL fate verification.
use worth_store_physical_format::PersistedPhysicalRecoveryProjection;

use super::super::*;
use crate::physical_runtime::{SharedRecoveryCheckpoint, StoreRecoveryBindingFreshnessSample};

// Drop order mirrors the original local scope on unwind: WAL fingerprint,
// projection and freshness sample release before `media` drops its lease.
pub(super) struct FirstPassObservation<'claim> {
    pub(super) inventory: SelectedWalMediaFingerprint,
    pub(super) projection: PersistedPhysicalRecoveryProjection,
    pub(super) sample: StoreRecoveryBindingFreshnessSample,
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) first_heads: head_v14::ObservedHeadV14<'claim>,
    pub(super) controls: controls::Controls,
    pub(super) selected_tier: Option<tier::selection::ObservedTierSelection>,
    pub(super) selected: selection::Selection,
}

#[inline(never)]
pub(super) fn observe<'claim>(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &'claim VerifiedPendingWalReleaseCustody,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
    checkpoint: &SharedRecoveryCheckpoint,
) -> Result<Box<FirstPassObservation<'claim>>, Denial> {
    let store = media.store_identity();
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = selection::observe(&mut discovery, reopen, claim, checkpoint.stream())?;
    let selected_tier = match tier_claim {
        Some(tier_claim) => Some(tier::selection::observe(
            &mut discovery,
            store,
            reopen,
            tier_claim,
            checkpoint.stream(),
        )?),
        None if selected.root.tier_epoch_anchor().is_none() => None,
        None => return Err(Denial::RootBinding),
    };
    if selected_tier
        .as_ref()
        .is_some_and(|tier| tier.root() != &selected.root || tier.free_header() != &selected.free)
    {
        return Err(Denial::RootBinding);
    }
    let controls = controls::observe(
        &mut discovery,
        &selected,
        reopen.format(),
        claim,
        recovery_allocation,
        checkpoint.stream(),
    )?;
    let first_heads = head_v14::observe(
        &mut discovery,
        &mut crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(
            coordination,
        )
        .map_err(Denial::WalReadOwnership)?,
        &selected,
        &controls,
        claim,
        effective,
        recovery_allocation,
        reopen.format(),
    )?;
    let inventory = wal_inventory::admit_complete_inventory(&mut discovery, coordination).map_err(
        |denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission,
            )
        },
    )?;
    if let Some(tier) = tier_claim {
        tier::wal::verify(&inventory, tier)?;
    }
    let media = discovery.finish();
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        crate::physical_runtime::StoreRecoverySamplingBasis::Checkpoint(claim.checkpoint()),
        IntegrityAdmittedRecoveryWalFrameView::from_frames(inventory.frames()),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(Denial::binding_sampling)?;
    if !wal_fate::matches_pending_release(
        media.store_identity(),
        claim,
        &sample,
        inventory.frames(),
        reopen.format(),
    ) {
        return Err(Denial::WalFate);
    }
    if let Some(tier) = tier_claim {
        tier::wal::verify_sample(&sample, tier)?;
    }
    verify_selected_fate_batches(&media, claim, &controls, &sample, &inventory)?;
    let projection = wal_fate::matched_pending_projection(claim, &sample, reopen.format())
        .ok_or(Denial::WalFate)?;
    let inventory = inventory.into_fingerprint();
    Ok(Box::new(FirstPassObservation {
        media,
        selected,
        selected_tier,
        controls,
        first_heads,
        inventory,
        sample,
        projection,
    }))
}

#[inline(never)]
fn verify_selected_fate_batches(
    media: &AdmittedRecoveryFilesystemMedia,
    claim: &VerifiedPendingWalReleaseCustody,
    controls: &controls::Controls,
    sample: &StoreRecoveryBindingFreshnessSample,
    inventory: &wal_inventory::AdmittedWalInventory,
) -> Result<(), Denial> {
    let cutoff = claim
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    if let Some(base) = claim.selected_release() {
        for batch in base.batches() {
            let attempt = controls
                .selected_base_attempt(batch.descriptor_record())
                .ok_or(Denial::ControlFrame)?;
            let tip = batch
                .tip_provenance()
                .map_err(|_| Denial::CertificateRoster)?;
            if !wal_fate::matches_selected_fate(
                media.store_identity(),
                attempt,
                tip,
                sample,
                inventory.frames(),
                cutoff,
            ) {
                return Err(Denial::WalFate);
            }
        }
    }
    if let Some(base) = claim.selected_head_v2() {
        let joined = controls
            .selected_head_v2_controls()
            .ok_or(Denial::ControlFrame)?;
        for batch in base.batches() {
            let attempt = joined
                .attempt_for_descriptor(batch.descriptor_record())
                .ok_or(Denial::ControlFrame)?;
            let tip = batch
                .tip_provenance()
                .map_err(|_| Denial::CertificateRoster)?;
            if !wal_fate::matches_selected_fate(
                media.store_identity(),
                attempt,
                tip,
                sample,
                inventory.frames(),
                cutoff,
            ) {
                return Err(Denial::WalFate);
            }
        }
    }
    if let Some(base) = claim.addressed_release_base() {
        let selected_base = controls.selected_base().ok_or(Denial::ControlFrame)?;
        for tip in base
            .batches()
            .iter()
            .map(|batch| {
                batch
                    .tip_provenance()
                    .map_err(|_| Denial::CertificateRoster)
            })
            .chain(
                std::iter::once(Ok(base.accumulator().tip())).filter(|_| base.batches().is_empty()),
            )
        {
            let tip = tip?;
            let attempt = selected_base
                .attempt_for_descriptor(tip.descriptor_record())
                .ok_or(Denial::ControlFrame)?;
            if !wal_fate::matches_selected_fate(
                media.store_identity(),
                attempt,
                tip,
                sample,
                inventory.frames(),
                cutoff,
            ) {
                return Err(Denial::WalFate);
            }
        }
    }
    Ok(())
}
