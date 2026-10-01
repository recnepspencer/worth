//! Store-owned second walk of the checkpoint-source and post-WAL head trees.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, verify_release_custody_head_controls,
    verify_release_custody_head_successor, BlobRecordV1, ReleaseCustodyHeadControlIdentityV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadRosterDigestV1, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedPendingWalReleaseCustody,
    VerifiedSelectedReleaseHeadReplayV14,
};

use super::super::{
    control_frames::SelectedControlMediaFingerprint, release_heads,
    SelectedMediaRejoinDenial as Denial,
};
use super::{controls::Controls, head_effect_media, selection::Selection};
use crate::physical_runtime::{
    durability::ReleaseHeadCapacityCharge, PhysicalRecoveryAllocationAdmission,
};

#[path = "head_v14/inverse.rs"]
mod inverse;

pub(super) struct ObservedHeadV14<'claim> {
    checkpoint: release_heads::ObservedReleaseHeads,
    pre_pending: release_heads::ObservedReleaseHeads,
    effective: release_heads::ObservedReleaseHeads,
    effect: head_effect_media::ObservedReleaseHeadEffectV14<'claim>,
}

impl ObservedHeadV14<'_> {
    pub(super) fn replay(&self) -> &VerifiedSelectedReleaseHeadReplayV14 {
        self.effect.replay()
    }

    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        self.checkpoint
            .owned_heap_bytes()?
            .checked_add(self.pre_pending.owned_heap_bytes()?)?
            .checked_add(self.effective.owned_heap_bytes()?)?
            .checked_add(self.effect.owned_heap_bytes()?)
    }

    pub(super) fn same_bytes(&self, other: &Self) -> bool {
        self.checkpoint.same_bytes(&other.checkpoint)
            && self.pre_pending.same_bytes(&other.pre_pending)
            && self.effective.same_bytes(&other.effective)
            && self.effect.same_bytes(&other.effect)
    }

    pub(super) fn into_fingerprint(self) -> SelectedControlMediaFingerprint {
        let mut fingerprint = self.checkpoint.into_fingerprint();
        fingerprint.extend(self.pre_pending.into_fingerprint());
        fingerprint.extend(self.effective.into_fingerprint());
        fingerprint.extend(self.effect.into_fingerprint());
        fingerprint
    }
}

pub(super) fn observe<'claim>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &Selection,
    controls: &Controls,
    claim: &'claim VerifiedPendingWalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    allocation: PhysicalRecoveryAllocationAdmission,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Result<ObservedHeadV14<'claim>, Denial> {
    let replay = claim
        .selected_head_replay()
        .ok_or(Denial::CertificateRoster)?;
    let effect = replay.effect();
    let checkpoint_ref = selected.checkpoint_source_root.release_custody_head_root();
    let source_ref = selected.source_root.release_custody_head_root();
    let result_ref = selected.root.release_custody_head_root();
    if checkpoint_ref != effective.checkpoint_source_root()
        || selected
            .checkpoint_source_root
            .next_release_custody_head_block()
            != effective.checkpoint_source_next_block()
        || source_ref != effect.source_root()
        || selected.source_root.next_release_custody_head_block() != effect.source_next_block()
        || result_ref != Some(effective.effective_root())
        || result_ref != Some(effect.result_root())
        || selected.root.next_release_custody_head_block() != effective.effective_next_block()
        || selected.root.next_release_custody_head_block() != effect.result_next_block()
    {
        return Err(Denial::CertificateRoster);
    }
    let page = u64::from(format.page_size().bytes());
    let checkpoint_count = effective.checkpoint_source_heads().len() as u64;
    let result_count = effective.effective_heads().len() as u64;
    let inverse =
        inverse::InversePendingUpsert::new(effective.effective_heads(), effect.mutation())?;
    let pre_pending_count = inverse.count();
    let closure = ReleaseHeadCapacityCharge::selected_roster_closure_bytes(checkpoint_count, page)
        .and_then(|bytes| {
            bytes.checked_add(ReleaseHeadCapacityCharge::selected_roster_closure_bytes(
                pre_pending_count,
                page,
            )?)
        })
        .and_then(|bytes| {
            bytes.checked_add(ReleaseHeadCapacityCharge::selected_roster_closure_bytes(
                result_count,
                page,
            )?)
        })
        .and_then(|bytes| bytes.checked_add(effect.framed_bytes()?))
        .and_then(|bytes| bytes.checked_add(BLOB_CONTROL_FRAME_MAX_BYTES as u64 * 4))
        .ok_or(Denial::BoundExceeded)?;
    if closure > allocation.byte_limit()
        || allocation.store_identity() != discovery.store_identity()
    {
        return Err(Denial::BoundExceeded);
    }
    // This is the head-replay observation window, not a claim that the
    // surrounding C8 state or Store's other observations have been charged.
    // The retained replay and recomputation scratch coexist during verify.
    let head_ceiling = allocation
        .byte_limit()
        .checked_sub(replay.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .ok_or(Denial::BoundExceeded)?;
    let checkpoint_digest = if let Some(base) = claim.selected_head_v2() {
        if base.checkpoint_source_root() != &selected.checkpoint_source_root
            || base.checkpoint_source_root().release_custody_head_root() != checkpoint_ref
            || base
                .checkpoint_source_root()
                .next_release_custody_head_block()
                != selected
                    .checkpoint_source_root
                    .next_release_custody_head_block()
            || base.accumulator_v2().head_count() != checkpoint_count
        {
            return Err(Denial::CertificateRoster);
        }
        base.accumulator_v2().head_roster_digest()
    } else if claim.marker().is_some() && checkpoint_count == 0 && checkpoint_ref.is_none() {
        ReleaseCustodyHeadRosterDigestV1::new(None, 0).finish().1
    } else {
        return Err(Denial::CertificateRoster);
    };
    let checkpoint = release_heads::observe(
        discovery,
        &selected.checkpoint_source_root,
        format,
        allocation,
        checkpoint_count,
        checkpoint_digest,
        head_ceiling,
    )?;
    if checkpoint.entries() != effective.checkpoint_source_heads() {
        return Err(Denial::CertificateRoster);
    }
    let checkpoint_heap = checkpoint.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
    let pre_pending_window = head_ceiling
        .checked_sub(checkpoint_heap)
        .ok_or(Denial::BoundExceeded)?;
    let (expected_count, expected_digest) = inverse.digest(source_ref)?;
    if expected_count != pre_pending_count {
        return Err(Denial::CertificateRoster);
    }
    let pre_pending = release_heads::observe(
        discovery,
        &selected.source_root,
        format,
        allocation,
        pre_pending_count,
        expected_digest,
        pre_pending_window,
    )?;
    if !inverse.matches(pre_pending.entries()) {
        return Err(Denial::CertificateRoster);
    }
    let effect_window = pre_pending_window
        .checked_sub(
            pre_pending
                .owned_heap_bytes()
                .ok_or(Denial::BoundExceeded)?,
        )
        .ok_or(Denial::BoundExceeded)?;
    let effect_observed = head_effect_media::observe(
        discovery,
        replay,
        &selected.source_root,
        &selected.root,
        format,
        effect_window,
    )?;
    let result_window = effect_window
        .checked_sub(
            effect_observed
                .owned_heap_bytes()
                .ok_or(Denial::BoundExceeded)?,
        )
        .ok_or(Denial::BoundExceeded)?;
    let result = release_heads::observe(
        discovery,
        &selected.root,
        format,
        allocation,
        result_count,
        effective.effective_digest(),
        result_window,
    )?;
    if result.entries() != effective.effective_heads() {
        return Err(Denial::CertificateRoster);
    }
    // The bounded V3 manifest decode and canonical control re-encodings run
    // while all three rosters and effect witnesses remain live. Four maximum
    // control frames cover decoded identities, payload/frame encodings and
    // the source-basis encoder; this is scratch, not retained control bytes.
    if result_window
        .checked_sub(result.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .is_none_or(|bytes| bytes < 4 * BLOB_CONTROL_FRAME_MAX_BYTES as u64)
    {
        return Err(Denial::BoundExceeded);
    }
    verify_pending_upsert(controls, claim, &pre_pending)?;
    Ok(ObservedHeadV14 {
        checkpoint,
        pre_pending,
        effective: result,
        effect: effect_observed,
    })
}

fn verify_pending_upsert(
    controls: &Controls,
    claim: &VerifiedPendingWalReleaseCustody,
    pre_pending: &release_heads::ObservedReleaseHeads,
) -> Result<(), Denial> {
    let replay = claim
        .selected_head_replay()
        .ok_or(Denial::CertificateRoster)?;
    let ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior,
        next,
    } = replay.effect().mutation()
    else {
        return Err(Denial::CertificateRoster);
    };
    let (descriptor_bytes, reservation_bytes, manifest_bytes) = controls.pending_controls();
    let (
        Ok(BlobRecordV1::ReclaimDescriptorV3(descriptor)),
        Ok(BlobRecordV1::OriginalDropReserved(reservation)),
        Ok(BlobRecordV1::DropSetManifestV3(manifest)),
    ) = (
        decode_blob_record(descriptor_bytes),
        decode_blob_record(reservation_bytes),
        decode_blob_record(manifest_bytes),
    )
    else {
        return Err(Denial::ControlFrame);
    };
    let frames = ReleaseCustodyHeadControlIdentityV1::new(
        claim.descriptor_record(),
        claim.descriptor_frame_sha256(),
        claim.reservation_record(),
        claim.reservation_frame_sha256(),
        claim.manifest_record(),
        claim.manifest_frame_sha256(),
    )
    .map_err(|_| Denial::ControlFrame)?;
    match expected_prior {
        Some(prior) => verify_release_custody_head_successor(
            prior,
            next,
            frames,
            descriptor,
            reservation,
            &manifest,
        )
        .map_err(|_| Denial::ControlFrame)?,
        None => {
            verify_release_custody_head_controls(next, frames, descriptor, reservation, &manifest)
                .map_err(|_| Denial::ControlFrame)?;
            if next.cumulative_dropped() != u64::from(manifest.count()) {
                return Err(Denial::ControlFrame);
            }
        }
    }
    let prior = pre_pending
        .entries()
        .binary_search_by_key(&next.key(), |entry| entry.key())
        .ok()
        .and_then(|index| pre_pending.entries().get(index))
        .copied();
    if prior != expected_prior {
        return Err(Denial::ControlFrame);
    }
    Ok(())
}
