//! The selected route's declared tier must agree with its durable arena ID
//! under the validated free-space header epoch, not merely its frame bytes.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    arena_tier_at_epoch, decode_checkpoint_certificate, CheckpointCertificateKind,
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, PhysicalTierClass,
    RecordArtifactFile, TierEpochActivationV1, TierEpochCheckpointCertificateV1,
    TierEpochWalFrameWitnessV1,
};
use worth_store_recovery_physics::{
    SelectedTierEpochCustodySource, VerifiedSelectedTierEpochCustody,
};

use super::{PageObservationFailure, TierEvidence};

pub(super) fn validate(
    generation: u64,
    placements: &[CurrentPhysicalRecordPlacement],
    header: &DurableFreeSpaceManifestHeader,
    root_anchor: Option<[u8; 32]>,
    evidence: TierEvidence<'_>,
) -> Result<Option<VerifiedSelectedTierEpochCustody>, PageObservationFailure> {
    let invalid = || PageObservationFailure::InvalidManifest {
        target: None,
        artifact: RecordArtifactFile::FreeSpaceManifest { generation },
    };
    if header.tier_epoch_start().is_some() != root_anchor.is_some() {
        return Err(invalid());
    }
    let authorization =
        selected_epoch_authorization(header, root_anchor, &evidence).ok_or_else(invalid)?;
    for placement in placements {
        match placement {
            CurrentPhysicalRecordPlacement::Inline(inline)
                if inline.tier_class() != PhysicalTierClass::Primary =>
            {
                return Err(invalid());
            }
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.arena_range().arena().get() >= header.next_arena()
                    || extent.tier_class()
                        != arena_tier_at_epoch(
                            header.tier_epoch_start(),
                            extent.arena_range().arena(),
                        ) =>
            {
                return Err(invalid());
            }
            _ => {}
        }
    }
    authorization
        .map(|basis| {
            VerifiedSelectedTierEpochCustody::admit_selected_tier(
                evidence.selection,
                evidence.checkpoint.ok_or_else(invalid)?,
                header,
                basis.intent,
                basis.intent_frame,
                basis.completed_frame,
                basis.source,
            )
            .map_err(|_| invalid())
        })
        .transpose()
}

struct TierAuthorization {
    intent: TierEpochActivationV1,
    intent_frame: TierEpochWalFrameWitnessV1,
    completed_frame: TierEpochWalFrameWitnessV1,
    source: SelectedTierEpochCustodySource,
}

fn selected_epoch_authorization(
    header: &DurableFreeSpaceManifestHeader,
    anchor: Option<[u8; 32]>,
    evidence: &TierEvidence<'_>,
) -> Option<Option<TierAuthorization>> {
    let Some(anchor) = anchor else {
        // An unresolved selected activation must not silently reopen an
        // unanchored namespace for new allocation.
        return evidence
            .sample
            .tier_epoch_activation()
            .is_none()
            .then_some(None);
    };
    let Some(epoch) = header.tier_epoch_start() else {
        return None;
    };
    let checkpoint = evidence.selection.checkpoint()?;
    let stream = evidence.checkpoint?;
    if stream.facts() != *checkpoint.checkpoint() {
        return None;
    }
    let selected = evidence.selection.root().selected();
    let selected_root = selected.manifest();
    let root_sha256: [u8; 32] =
        Sha256::digest(selected_root.encode(selected.selector().format())).into();
    let frames = evidence.selected_wal.iter().map(|frame| {
        TierEpochWalFrameWitnessV1::new(
            frame.lsn_start(),
            frame.lsn_end(),
            frame.identity_digest(),
            frame.payload_digest(),
        )
    });
    // Preserve the former Option collection's fail-closed validation, including
    // malformed witnesses unrelated to the selected activation interval.
    if frames.clone().any(|frame| frame.is_none()) {
        return None;
    }
    let mut certificate = None;
    for frame in stream.certificate_records() {
        let Ok((kind, payload)) = decode_checkpoint_certificate(frame) else {
            return None;
        };
        if kind == CheckpointCertificateKind::TierEpoch {
            let Ok(decoded) = TierEpochCheckpointCertificateV1::decode(payload) else {
                return None;
            };
            if certificate.replace(decoded).is_some() {
                return None;
            }
        }
    }
    match certificate {
        Some(cert) => {
            let intent = cert.intent();
            let retained_activation_agrees = match evidence.sample.tier_epoch_activation() {
                None => true,
                Some(observed) => selected_pair_matches_certificate(
                    frames.clone(),
                    intent,
                    cert.intent_frame(),
                    cert.completed_frame(),
                    observed.intent(),
                    (
                        observed.intent_range().start().get(),
                        observed.intent_range().end_exclusive().get(),
                    ),
                    observed
                        .completion_range()
                        .map(|range| (range.start().get(), range.end_exclusive().get())),
                ),
            };
            let valid = cert.checkpoint() == checkpoint.checkpoint().source().identity()
                && cert.root_generation() == checkpoint.checkpoint().source().root().generation()
                && cert.root_sha256() == checkpoint.source_root_frame_sha256()
                && cert.anchor() == anchor
                && intent.epoch_anchor() == anchor
                && intent.tier_epoch_start() == epoch
                && intent.store()
                    == checkpoint
                        .checkpoint()
                        .source()
                        .identity()
                        .store_identity()
                        .bytes()
                && selected_root.generation() >= cert.root_generation()
                && (selected_root.generation() != intent.candidate_root_generation()
                    || root_sha256 == intent.candidate_root_sha256())
                && cert.completed_frame().lsn_end_exclusive()
                    <= checkpoint
                        .checkpoint()
                        .compaction_cutover()
                        .wal_cutoff_lsn_exclusive()
                // A retained pre-cutoff copy of the already-folded pair is
                // allowed only if both exact selected C9 frame witnesses
                // match. Any later, duplicate, partial, or substituted pair
                // remains a second attempt and is denied.
                && retained_activation_agrees
                && retained_witness_agrees(frames.clone(), cert.intent_frame())
                && retained_witness_agrees(frames.clone(), cert.completed_frame());
            valid.then_some(Some(TierAuthorization {
                intent,
                intent_frame: cert.intent_frame(),
                completed_frame: cert.completed_frame(),
                source: SelectedTierEpochCustodySource::SelectedCheckpointCertificate,
            }))
        }
        None => {
            let Some(observed) = evidence.sample.tier_epoch_activation() else {
                return None;
            };
            let Some(completed_range) = observed.completion_range() else {
                return None;
            };
            let intent = observed.intent();
            let intent_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
            let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
            let intent_frame = exact_selected_frame(
                frames.clone(),
                observed.intent_range().start().get(),
                observed.intent_range().end_exclusive().get(),
                intent_digest,
            )?;
            let completed_frame = exact_selected_frame(
                frames.clone(),
                completed_range.start().get(),
                completed_range.end_exclusive().get(),
                completed_digest,
            )?;
            let valid = intent.epoch_anchor() == anchor
                && intent.tier_epoch_start() == epoch
                && intent.store()
                    == checkpoint
                        .checkpoint()
                        .source()
                        .identity()
                        .store_identity()
                        .bytes()
                && selected_root.generation() == intent.candidate_root_generation()
                && root_sha256 == intent.candidate_root_sha256()
                && intent_frame.lsn_end_exclusive() <= completed_frame.lsn_start();
            valid.then_some(Some(TierAuthorization {
                intent,
                intent_frame,
                completed_frame,
                source: SelectedTierEpochCustodySource::SelectedWalPair,
            }))
        }
    }
}

fn selected_pair_matches_certificate(
    frames: impl Iterator<Item = Option<TierEpochWalFrameWitnessV1>> + Clone,
    intent: TierEpochActivationV1,
    intent_frame: TierEpochWalFrameWitnessV1,
    completed_frame: TierEpochWalFrameWitnessV1,
    observed_intent: TierEpochActivationV1,
    observed_intent_range: (u64, u64),
    observed_completed_range: Option<(u64, u64)>,
) -> bool {
    let Some((completed_start, completed_end)) = observed_completed_range else {
        return false;
    };
    let intent_digest: [u8; 32] = Sha256::digest(intent.encode()).into();
    let completed_digest: [u8; 32] = Sha256::digest(intent.completed().encode()).into();
    observed_intent == intent
        && exact_selected_frame(
            frames.clone(),
            observed_intent_range.0,
            observed_intent_range.1,
            intent_digest,
        ) == Some(intent_frame)
        && exact_selected_frame(frames, completed_start, completed_end, completed_digest)
            == Some(completed_frame)
}

fn exact_selected_frame(
    frames: impl Iterator<Item = Option<TierEpochWalFrameWitnessV1>>,
    start: u64,
    end: u64,
    payload_digest: [u8; 32],
) -> Option<TierEpochWalFrameWitnessV1> {
    let mut selected = None;
    for frame in frames {
        let frame = frame?;
        if frame.lsn_start() < end && start < frame.lsn_end_exclusive() {
            if selected.is_some()
                || frame.lsn_start() != start
                || frame.lsn_end_exclusive() != end
                || frame.payload_digest() != payload_digest
            {
                return None;
            }
            selected = Some(frame);
        }
    }
    selected
}

fn retained_witness_agrees(
    frames: impl Iterator<Item = Option<TierEpochWalFrameWitnessV1>>,
    witness: TierEpochWalFrameWitnessV1,
) -> bool {
    let mut found = false;
    for frame in frames {
        let Some(frame) = frame else {
            return false;
        };
        if frame.lsn_start() < witness.lsn_end_exclusive()
            && witness.lsn_start() < frame.lsn_end_exclusive()
        {
            if found
                || frame.lsn_start() != witness.lsn_start()
                || frame.lsn_end_exclusive() != witness.lsn_end_exclusive()
                || frame.identity_digest() != witness.identity_digest()
                || frame.payload_digest() != witness.payload_digest()
            {
                return false;
            }
            found = true;
        }
    }
    // The selected checkpoint itself transfers custody of compacted frames.
    // Any still-retained frame must agree byte-for-byte with that witness.
    true
}

#[cfg(test)]
mod tests;
