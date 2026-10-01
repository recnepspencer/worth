//! The selected release roster is a set of physical controls, not only its tip.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordV1, DropSetManifestV3,
    PersistedRecordIdentity, ReleaseCheckpointAccumulatorV1, ReleaseCheckpointBatchV1,
    ReleasedDropCumulativeEvidenceV1,
};
use worth_store_recovery_physics::{
    VerifiedAddressedCheckpointReleaseBase, VerifiedSelectedCheckpointCustody,
};

use super::{
    super::SelectedMediaRejoinDenial as Denial, ObservedSelectedControlFrame,
    ObservedSelectedControls,
};

pub(super) fn attempt_for_descriptor(
    controls: &ObservedSelectedControls,
    record: PersistedRecordIdentity,
) -> Option<[u8; 16]> {
    [
        &controls.descriptor,
        &controls.reservation,
        &controls.manifest,
    ]
    .into_iter()
    .chain(controls.additional.iter())
    .find(|frame| frame.record() == record)
    .and_then(|frame| match decode_blob_record(frame.bytes()).ok()? {
        BlobRecordV1::ReclaimDescriptorV3(value) => Some(value.base().reclaim_attempt()),
        _ => None,
    })
}

pub(super) fn predecessor_controls(
    controls: &ObservedSelectedControls,
    descriptor_record: PersistedRecordIdentity,
    descriptor_sha256: [u8; 32],
) -> Option<(BlobReclaimDescriptorV3, DropSetManifestV3)> {
    let frames = [
        &controls.descriptor,
        &controls.reservation,
        &controls.manifest,
    ]
    .into_iter()
    .chain(controls.additional.iter());
    decode_predecessor_frames(
        frames.map(|frame| (frame.record(), frame.frame_sha256(), frame.bytes())),
        descriptor_record,
        descriptor_sha256,
    )
}

pub(super) fn latest_for_source(
    controls: &ObservedSelectedControls,
    source: worth_store_physical_format::BlobReclaimSourceBasisV1,
) -> Option<(PersistedRecordIdentity, [u8; 32])> {
    let mut latest = None;
    for frame in [
        &controls.descriptor,
        &controls.reservation,
        &controls.manifest,
    ]
    .into_iter()
    .chain(controls.additional.iter())
    {
        if frame.kind() != worth_store_physical_format::BlobRecordKind::ReclaimDescriptorV3 {
            continue;
        }
        let Some((descriptor, manifest)) =
            predecessor_controls(controls, frame.record(), frame.frame_sha256())
        else {
            return None;
        };
        if manifest.source_basis() != source {
            continue;
        }
        let generation = descriptor.base().candidate_root_generation();
        match latest {
            Some((prior_generation, _, _)) if prior_generation == generation => return None,
            Some((prior_generation, _, _)) if prior_generation > generation => {}
            _ => latest = Some((generation, frame.record(), frame.frame_sha256())),
        }
    }
    latest.map(|(_, record, sha)| (record, sha))
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn decode_predecessor_frames<
    'a,
>(
    mut frames: impl Iterator<Item = (PersistedRecordIdentity, [u8; 32], &'a [u8])> + Clone,
    descriptor_record: PersistedRecordIdentity,
    descriptor_sha256: [u8; 32],
) -> Option<(BlobReclaimDescriptorV3, DropSetManifestV3)> {
    let (_, _, descriptor_bytes) = frames
        .clone()
        .find(|(record, sha, _)| *record == descriptor_record && *sha == descriptor_sha256)?;
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(descriptor_bytes).ok()?
    else {
        return None;
    };
    let (_, _, manifest_bytes) = frames.find(|(record, sha, _)| {
        *record == descriptor.base().manifest_record()
            && *sha == descriptor.base().manifest_frame_sha256()
    })?;
    let BlobRecordV1::DropSetManifestV3(manifest) = decode_blob_record(manifest_bytes).ok()? else {
        return None;
    };
    Some((descriptor, manifest))
}

pub(super) fn verify(
    claim: &VerifiedSelectedCheckpointCustody,
    frames: &BTreeMap<PersistedRecordIdentity, &ObservedSelectedControlFrame>,
) -> Result<(), Denial> {
    verify_roster(
        claim.batches(),
        claim.accumulator(),
        claim.tip_custody_digest(),
        claim.checkpoint().source().root().generation(),
        frames,
    )
}

pub(super) fn verify_addressed(
    claim: &VerifiedAddressedCheckpointReleaseBase,
    frames: &BTreeMap<PersistedRecordIdentity, &ObservedSelectedControlFrame>,
) -> Result<(), Denial> {
    verify_roster(
        claim.batches(),
        claim.accumulator(),
        claim.tip_descriptor().custody_digest(),
        claim.checkpoint_root().generation(),
        frames,
    )
}

fn verify_roster(
    batches: &[ReleaseCheckpointBatchV1],
    accumulator: ReleaseCheckpointAccumulatorV1,
    tip_custody_digest: [u8; 32],
    checkpoint_source_generation: u64,
    frames: &BTreeMap<PersistedRecordIdentity, &ObservedSelectedControlFrame>,
) -> Result<(), Denial> {
    let mut descriptors = BTreeMap::new();
    let mut reservations = BTreeMap::new();
    let mut reservations_by_source = BTreeMap::new();
    let mut manifests = BTreeMap::new();
    for (&record, frame) in frames {
        match decode_blob_record(frame.bytes()).map_err(|_| Denial::ControlFrame)? {
            BlobRecordV1::ReclaimDescriptorV3(value) => {
                descriptors.insert(record, (value, frame.frame_sha256()));
            }
            BlobRecordV1::OriginalDropReserved(value) => {
                reservations_by_source
                    .entry((
                        value.store(),
                        value.reclaim_attempt(),
                        value.manifest_record(),
                    ))
                    .or_insert_with(Vec::new)
                    .push(record);
                reservations.insert(record, (value, frame.frame_sha256()));
            }
            BlobRecordV1::DropSetManifestV3(value) => {
                manifests.insert(record, (value, frame.frame_sha256()));
            }
            _ => return Err(Denial::ControlFrame),
        }
    }
    let mut used_reservations = BTreeSet::new();
    let mut used_manifests = BTreeSet::new();
    for descriptor in descriptors.values().map(|(value, _)| value) {
        let base = descriptor.base();
        let Some((manifest, digest)) = manifests.get(&base.manifest_record()) else {
            return Err(Denial::ControlFrame);
        };
        if *digest != base.manifest_frame_sha256()
            || manifest.count() != base.manifest_count()
            || manifest.store() != base.store()
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_basis_digest() != base.source_basis_digest()
        {
            return Err(Denial::ControlFrame);
        }
        if !used_manifests.insert(base.manifest_record()) {
            return Err(Denial::ControlFrame);
        }
        let Some(matching) = reservations_by_source.get(&(
            base.store(),
            base.reclaim_attempt(),
            base.manifest_record(),
        )) else {
            return Err(Denial::ControlFrame);
        };
        if matching.len() != 1 || !used_reservations.insert(matching[0]) {
            return Err(Denial::ControlFrame);
        }
        let reservation = reservations
            .get(&matching[0])
            .ok_or(Denial::ControlFrame)?
            .0;
        if reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
            || reservation.source_basis_digest() != base.source_basis_digest()
            || reservation.request() != descriptor.custody().request()
        {
            return Err(Denial::ControlFrame);
        }
    }
    if used_reservations.len() != reservations.len() || used_manifests.len() != manifests.len() {
        return Err(Denial::ControlFrame);
    }

    let tip = accumulator.tip();
    let mut cumulative = (
        accumulator.prior_cumulative_dropped(),
        accumulator.prior_cumulative_digest(),
    );
    for batch in batches {
        verify_batch(*batch, &descriptors, &reservations)?;
        let descriptor = &descriptors
            .get(&batch.descriptor_record())
            .ok_or(Denial::ControlFrame)?
            .0;
        let manifest_count = manifests
            .get(&descriptor.base().manifest_record())
            .ok_or(Denial::ControlFrame)?
            .0
            .count();
        cumulative = fold_observed_batch(*batch, manifest_count, cumulative)?;
    }
    if !batches.is_empty()
        && cumulative
            != (
                accumulator.cumulative_dropped(),
                accumulator.cumulative_digest(),
            )
    {
        return Err(Denial::CertificateRoster);
    }
    let Some((tip_descriptor, tip_digest)) = descriptors.get(&tip.descriptor_record()) else {
        return Err(Denial::ControlFrame);
    };
    if *tip_digest != tip.descriptor_frame_sha256()
        || tip_descriptor.custody_digest() != tip_custody_digest
        || tip_descriptor.custody().request() != tip.request()
        || tip_descriptor.base().terminal() != accumulator.terminal()
        || tip_descriptor.base().candidate_root_generation() != tip.candidate_root_generation()
        || reservations
            .get(&tip.reservation_record())
            .is_none_or(|(value, digest)| {
                *digest != tip.reservation_frame_sha256()
                    || value.request() != tip.request()
                    || value.reclaim_attempt() != tip_descriptor.base().reclaim_attempt()
            })
    {
        return Err(Denial::ControlFrame);
    }

    let current = batches
        .iter()
        .map(|batch| batch.descriptor_record())
        .chain(std::iter::once(tip.descriptor_record()))
        .collect::<BTreeSet<_>>();
    for (&record, (descriptor, _)) in &descriptors {
        if !current.contains(&record)
            && !uncovered_descriptor_predates_checkpoint(
                descriptor.base().candidate_root_generation(),
                checkpoint_source_generation,
            )
        {
            return Err(Denial::CertificateRoster);
        }
    }

    // The accumulator is Store-wide, but a V3 predecessor is per released
    // generation. An older selected triple is validated and fingerprinted as
    // residue; only the exact current Batch or tip may supply custody.
    Ok(())
}

fn uncovered_descriptor_predates_checkpoint(
    candidate_generation: u64,
    checkpoint_source_generation: u64,
) -> bool {
    candidate_generation <= checkpoint_source_generation
}

fn fold_observed_batch(
    batch: ReleaseCheckpointBatchV1,
    observed_manifest_count: u16,
    prior: (u64, [u8; 32]),
) -> Result<(u64, [u8; 32]), Denial> {
    let folded = ReleasedDropCumulativeEvidenceV1::new(
        batch.descriptor_record(),
        batch.descriptor_frame_sha256(),
        batch.custody_digest(),
        batch.reservation_record(),
        batch.reservation_frame_sha256(),
        batch.fate(),
        batch.candidate_root_generation(),
        batch.candidate_root_sha256(),
        batch.predecessor(),
        observed_manifest_count,
        batch.terminal(),
    )
    .and_then(|step| step.advance(prior.0, prior.1))
    .map_err(|_| Denial::CertificateRoster)?;
    (folded == (batch.cumulative_dropped(), batch.cumulative_digest()))
        .then_some(folded)
        .ok_or(Denial::CertificateRoster)
}

#[cfg(test)]
#[path = "catalog/tests.rs"]
mod tests;

fn verify_batch(
    batch: ReleaseCheckpointBatchV1,
    descriptors: &BTreeMap<
        PersistedRecordIdentity,
        (
            worth_store_physical_format::BlobReclaimDescriptorV3,
            [u8; 32],
        ),
    >,
    reservations: &BTreeMap<
        PersistedRecordIdentity,
        (
            worth_store_physical_format::OriginalDropReservedV1,
            [u8; 32],
        ),
    >,
) -> Result<(), Denial> {
    let (descriptor, digest) = descriptors
        .get(&batch.descriptor_record())
        .ok_or(Denial::ControlFrame)?;
    let (reservation, reservation_digest) = reservations
        .get(&batch.reservation_record())
        .ok_or(Denial::ControlFrame)?;
    if *digest != batch.descriptor_frame_sha256()
        || *reservation_digest != batch.reservation_frame_sha256()
        || descriptor.custody_digest() != batch.custody_digest()
        || descriptor.custody().request() != batch.request()
        || reservation.request() != batch.request()
        || descriptor.base().reclaim_attempt() != reservation.reclaim_attempt()
        || descriptor.base().candidate_root_generation() != batch.candidate_root_generation()
        || descriptor.base().predecessor() != batch.predecessor()
        || descriptor.base().terminal() != batch.terminal()
    {
        return Err(Denial::ControlFrame);
    }
    Ok(())
}
