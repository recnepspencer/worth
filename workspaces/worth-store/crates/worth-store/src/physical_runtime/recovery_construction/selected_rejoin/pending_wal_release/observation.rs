//! Independent first/final selected-media observation and WAL fingerprint handoff.
use super::*;

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedPendingWalReleaseCustody,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedWalMediaFingerprint,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let checkpoint = coordination
        .require_selected_checkpoint(claim.checkpoint())
        .map_err(|_| Denial::CheckpointBinding)?;
    if tier_claim.is_some_and(|tier| {
        coordination
            .require_selected_checkpoint(tier.checkpoint())
            .is_err()
    }) {
        return Err(Denial::CheckpointBinding);
    }
    if claim.checkpoint().encoded_bytes() > MAX_CHECKPOINT_BYTES {
        return Err(Denial::BoundExceeded);
    }
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
        claim.checkpoint(),
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
                &sample,
                inventory.frames(),
                claim
                    .checkpoint()
                    .compaction_cutover()
                    .wal_cutoff_lsn_exclusive(),
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
                &sample,
                inventory.frames(),
                claim
                    .checkpoint()
                    .compaction_cutover()
                    .wal_cutoff_lsn_exclusive(),
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
                &sample,
                inventory.frames(),
                claim
                    .checkpoint()
                    .compaction_cutover()
                    .wal_cutoff_lsn_exclusive(),
            ) {
                return Err(Denial::WalFate);
            }
        }
    }
    let projection = wal_fate::matched_pending_projection(claim, &sample, reopen.format())
        .ok_or(Denial::WalFate)?;
    pause_before_final_reread();
    let mut final_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let final_selected =
        selection::observe(&mut final_discovery, reopen, claim, checkpoint.stream())?;
    let final_tier = match tier_claim {
        Some(tier_claim) => Some(tier::selection::observe(
            &mut final_discovery,
            store,
            reopen,
            tier_claim,
            checkpoint.stream(),
        )?),
        None => None,
    };
    if selected_tier.as_ref().is_some_and(|before| {
        !final_tier
            .as_ref()
            .is_some_and(|after| before.matches_reread(after))
    }) {
        return Err(Denial::RootBinding);
    }
    let final_controls = controls::observe(
        &mut final_discovery,
        &final_selected,
        reopen.format(),
        claim,
        recovery_allocation,
        checkpoint.stream(),
    )?;
    let final_heads = head_v14::observe(
        &mut final_discovery,
        &mut crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(
            coordination,
        )
        .map_err(Denial::WalReadOwnership)?,
        &final_selected,
        &final_controls,
        claim,
        effective,
        recovery_allocation,
        reopen.format(),
    )?;
    let final_inventory =
        wal_inventory::admit_complete_inventory(&mut final_discovery, coordination).map_err(
            |denial| {
                denial.at_resident_boundary(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission,
        )
            },
        )?;
    if let Some(tier) = tier_claim {
        tier::wal::verify(&final_inventory, tier)?;
    }
    if !selected.same_bytes(&final_selected)
        || !controls.same_bytes(&final_controls)
        || !first_heads.same_bytes(&final_heads)
    {
        return Err(Denial::ControlFrame);
    }
    if !inventory.matches_reread(&final_inventory) {
        return Err(Denial::WalFate);
    }
    // Only the final exact WAL observation remains live during historical
    // root walks. The first complete inventory was compared above, then
    // released before the shared recovery-memory budget is calculated.
    drop(selected_tier);
    drop(final_tier);
    drop(selected);
    drop(controls);
    drop(first_heads);
    drop(inventory);
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
        let (media, fingerprint) = ordered_walk::verify(
            final_discovery.finish(),
            &mut crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(
                coordination,
            )
            .map_err(Denial::WalReadOwnership)?,
            claim,
            effective,
            &final_controls,
            &sample,
            final_inventory.frames(),
            reopen.format(),
            retained,
        )?;
        (media, Some((fingerprint, retained)))
    } else {
        match claim.historical_batches() {
            [] => (final_discovery.finish(), None),
            [batch] => {
                let retained = delta::retained_memory(
                    &final_selected,
                    &final_controls,
                    claim,
                    &projection,
                    &sample,
                    checkpoint,
                )?;
                let (media, mut history_fingerprint) = historical_chain::verify_ordinary_parts(
                    final_discovery.finish(),
                    batch.chain(),
                    claim.checkpoint().source().root().generation(),
                    claim.source_root().generation(),
                    claim.source_root_sha256(),
                    &sample,
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
                    &sample,
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
                (media, Some((history_fingerprint, retained)))
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
