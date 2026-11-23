//! Independent first/final selected-media observation and WAL fingerprint handoff.
use super::*;

#[path = "observation/final_pass.rs"]
mod final_pass;
#[path = "observation/first_pass.rs"]
mod first_pass;
#[path = "observation/historical.rs"]
mod historical;

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
    let first = first_pass::observe(
        coordination,
        media,
        reopen,
        claim,
        recovery_allocation,
        effective,
        tier_claim,
        checkpoint,
    )?;
    pause_before_final_reread();
    let final_obs = final_pass::observe_and_match(
        coordination,
        first,
        reopen,
        claim,
        recovery_allocation,
        effective,
        tier_claim,
        checkpoint,
    )?;
    historical::verify_and_finalize(
        coordination,
        final_obs,
        reopen,
        claim,
        effective,
        checkpoint,
    )
}
