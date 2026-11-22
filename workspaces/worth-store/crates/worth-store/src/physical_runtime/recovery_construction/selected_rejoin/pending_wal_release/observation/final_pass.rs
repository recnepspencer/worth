//! Final reread observation and exact byte/fingerprint equality against first pass.
use worth_store_physical_format::PersistedPhysicalRecoveryProjection;

use super::super::*;
use super::first_pass::FirstPassObservation;
use crate::physical_runtime::{SharedRecoveryCheckpoint, StoreRecoveryBindingFreshnessSample};

pub(super) struct FinalPassObservation<'claim> {
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) final_selected: selection::Selection,
    pub(super) final_controls: controls::Controls,
    pub(super) final_heads: head_v14::ObservedHeadV14<'claim>,
    pub(super) final_inventory: wal_inventory::AdmittedWalInventory,
    pub(super) sample: StoreRecoveryBindingFreshnessSample,
    pub(super) projection: PersistedPhysicalRecoveryProjection,
}

#[inline(never)]
pub(super) fn observe_and_match<'claim>(
    coordination: &PhysicalRecoveryCoordination,
    first: Box<FirstPassObservation<'claim>>,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &'claim VerifiedPendingWalReleaseCustody,
    recovery_allocation: PhysicalRecoveryAllocationAdmission,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
    checkpoint: &SharedRecoveryCheckpoint,
) -> Result<Box<FinalPassObservation<'claim>>, Denial> {
    let FirstPassObservation {
        media,
        selected,
        selected_tier,
        controls,
        first_heads,
        inventory,
        sample,
        projection,
    } = *first;
    let store = media.store_identity();
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
    // root walks. The first inventory's fingerprint was compared above, then
    // released before the shared recovery-memory budget is calculated.
    drop(selected_tier);
    drop(final_tier);
    drop(selected);
    drop(controls);
    drop(first_heads);
    drop(inventory);
    Ok(Box::new(FinalPassObservation {
        media: final_discovery.finish(),
        final_selected,
        final_controls,
        final_heads,
        final_inventory,
        sample,
        projection,
    }))
}
