use super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverArtifactEvidence,
    RecoveryObserverManifestMembershipObservation, RecoveryObserverPageLsnObservation,
    RecoveryObserverResidueObservation, RecoveryObserverSelectorObservation,
};
use super::physical_format::{self, DurableFrame};

const ROOT_MANIFEST_KIND: u8 = 2;
const SEGMENT_MANIFEST_KIND: u8 = 5;
const EXTENT_MANIFEST_KIND: u8 = 6;
const FREE_SPACE_MANIFEST_KIND: u8 = 7;
const ROOT_ROUTING_KIND: u8 = 8;
const SEGMENT_MEMBERSHIP_KIND: u8 = 9;
const FREE_SPACE_MEMBERSHIP_KIND: u8 = 10;
const ROOT_SELECTOR_KIND: u8 = 11;

pub(super) fn observe(bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    let Some(frame) = physical_format::durable_frame(bytes) else {
        return physical_format::residue(bytes);
    };
    let mut evidence = RecoveryObserverArtifactEvidence {
        page_lsns: page_lsn_evidence(&frame),
        ..RecoveryObserverArtifactEvidence::empty()
    };
    let semantic = match frame.kind {
        ROOT_SELECTOR_KIND => selector_evidence(&frame),
        ROOT_MANIFEST_KIND => root_manifest_evidence(&frame),
        SEGMENT_MANIFEST_KIND => segment_manifest_evidence(&frame),
        EXTENT_MANIFEST_KIND => extent_manifest_evidence(&frame),
        FREE_SPACE_MANIFEST_KIND => free_space_manifest_evidence(&frame),
        ROOT_ROUTING_KIND => routing_block_evidence(&frame, 88, 72),
        SEGMENT_MEMBERSHIP_KIND => routing_block_evidence(&frame, 40, 56),
        FREE_SPACE_MEMBERSHIP_KIND => routing_block_evidence(&frame, 40, 56),
        _ => None,
    };
    if let Some((generation_links, selector, membership)) = semantic {
        evidence.generation_links = generation_links;
        evidence.selector = selector;
        evidence.manifest_membership = membership;
        evidence
    } else {
        evidence.residue = RecoveryObserverResidueObservation {
            bytes: bytes.len() as u64,
            digest: physical_format::digest_bytes(bytes),
        };
        evidence
    }
}

fn page_lsn_evidence(frame: &DurableFrame<'_>) -> RecoveryObserverPageLsnObservation {
    let mut digest = EvidenceDigestBuilder::new(b"worth.store.recovery-observer.page-lsn.v1");
    let mut record = Vec::with_capacity(17);
    record.push(frame.kind);
    record.extend_from_slice(&frame.identity.to_le_bytes());
    record.extend_from_slice(&frame.page_lsn.to_le_bytes());
    digest.record(&record);
    let finished = digest.finish();
    RecoveryObserverPageLsnObservation {
        count: finished.observations(),
        minimum: Some(frame.page_lsn),
        maximum: Some(frame.page_lsn),
        digest: finished.digest(),
    }
}

fn selector_evidence(
    frame: &DurableFrame<'_>,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() != 59
        || frame.identity == 0
        || frame.payload[..16] == [0; 16]
        || !matches!(frame.payload[16], 1 | 2)
        || frame.payload[17..25] == [0; 8]
        || frame.payload[51..] != [0; 8]
        || frame.payload[41..51] != frame_format(frame)
    {
        return None;
    }
    let linked_identity = physical_format::read_u64(frame.payload, 25)?;
    let linked_generation = physical_format::read_u64(frame.payload, 33)?;
    if (linked_identity == 0) != (linked_generation == 0) {
        return None;
    }
    let mut digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    let mut record = Vec::with_capacity(33);
    record.extend_from_slice(&frame.identity.to_le_bytes());
    record.extend_from_slice(&physical_format::read_u64(frame.payload, 17)?.to_le_bytes());
    record.extend_from_slice(&linked_identity.to_le_bytes());
    record.extend_from_slice(&linked_generation.to_le_bytes());
    digest.record(&record);
    Some((
        digest.finish(),
        Some(RecoveryObserverSelectorObservation {
            identity: frame.identity,
            linked_identity: (linked_identity != 0).then_some(linked_identity),
            store_identity: frame.payload[..16].try_into().ok()?,
            role: frame.payload[16],
            root_generation: physical_format::read_u64(frame.payload, 17)?,
        }),
        RecoveryObserverManifestMembershipObservation::empty(),
    ))
}

fn root_manifest_evidence(
    frame: &DurableFrame<'_>,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() != 320
        || frame.identity == 0
        || physical_format::read_u64(frame.payload, 0)? != frame.identity
        || frame.payload[18..24] != [0; 6]
        || frame.payload[41..48] != [0; 7]
        || frame.payload[121..128] != [0; 7]
        || frame.payload[156..160] != [0; 4]
        || frame.payload[161..168] != [0; 7]
        || frame.payload[233..240] != [0; 7]
        || frame.payload[297..304] != [0; 7]
    {
        return None;
    }
    let flag_offsets = [40, 120, 160, 232, 296];
    if flag_offsets.iter().any(|offset| frame.payload[*offset] > 1) {
        return None;
    }
    let mut digest =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    digest.record(&frame.payload[..40]);
    for offset in flag_offsets {
        if frame.payload[offset] == 1 {
            digest.record(&frame.payload[offset..reference_end(offset)]);
        }
    }
    let mut membership =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(frame.payload);
    Some((
        digest.finish(),
        None,
        RecoveryObserverManifestMembershipObservation {
            manifest_count: 1,
            member_count: flag_offsets
                .into_iter()
                .filter(|offset| frame.payload[*offset] == 1)
                .count() as u64,
            digest: membership.finish().digest(),
        },
    ))
}

fn segment_manifest_evidence(
    frame: &DurableFrame<'_>,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() < 24 || frame.payload[16..24] != [0; 8] || frame.identity == 0 {
        return None;
    }
    let count = usize::try_from(physical_format::read_u32(frame.payload, 12)?).ok()?;
    let expected = 24usize.checked_add(count.checked_mul(40)?)?;
    if count == 0 || frame.payload.len() != expected {
        return None;
    }
    let mut generation =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..16]);
    let mut membership =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(frame.payload);
    Some((
        generation.finish(),
        None,
        RecoveryObserverManifestMembershipObservation {
            manifest_count: 1,
            member_count: count as u64,
            digest: membership.finish().digest(),
        },
    ))
}

fn extent_manifest_evidence(
    frame: &DurableFrame<'_>,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() != 56 || frame.payload[48..] != [0; 8] || frame.identity == 0 {
        return None;
    }
    let mut generation =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..48]);
    Some((
        generation.finish(),
        None,
        RecoveryObserverManifestMembershipObservation {
            manifest_count: 1,
            member_count: 1,
            digest: physical_format::digest_bytes(frame.payload),
        },
    ))
}

fn free_space_manifest_evidence(
    frame: &DurableFrame<'_>,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() != 128
        || frame.identity == 0
        || physical_format::read_u64(frame.payload, 0)? != frame.identity
        || frame.payload[22..24] != [0; 2]
        || frame.payload[65..72] != [0; 7]
        || frame.payload[64] > 1
    {
        return None;
    }
    let mut generation =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..64]);
    let mut membership =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(frame.payload);
    let member_count =
        physical_format::read_u64(frame.payload, 24)?.saturating_add(u64::from(frame.payload[64]));
    Some((
        generation.finish(),
        None,
        RecoveryObserverManifestMembershipObservation {
            manifest_count: 1,
            member_count,
            digest: membership.finish().digest(),
        },
    ))
}

fn routing_block_evidence(
    frame: &DurableFrame<'_>,
    leaf_width: usize,
    branch_width: usize,
) -> Option<(
    RecoveryObserverEvidenceDigest,
    Option<RecoveryObserverSelectorObservation>,
    RecoveryObserverManifestMembershipObservation,
)> {
    if frame.payload.len() < 40
        || frame.identity == 0
        || frame.payload[21..24] != [0; 3]
        || frame.payload[32..40] != [0; 8]
        || physical_format::read_u64(frame.payload, 0)? == 0
        || physical_format::read_u64(frame.payload, 8)? != frame.identity
        || physical_format::read_u64(frame.payload, 24)? == 0
    {
        return None;
    }
    let count = usize::from(physical_format::read_u16(frame.payload, 18)?);
    if count == 0 {
        return None;
    }
    let width = match (
        frame.payload[20],
        physical_format::read_u16(frame.payload, 16)?,
    ) {
        (1, 0) => leaf_width,
        (2, level) if level != 0 => branch_width,
        _ => return None,
    };
    if frame.payload.len() != 40usize.checked_add(count.checked_mul(width)?)? {
        return None;
    }
    let mut generation =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..32]);
    let mut membership =
        EvidenceDigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(frame.payload);
    Some((
        generation.finish(),
        None,
        RecoveryObserverManifestMembershipObservation {
            manifest_count: 1,
            member_count: count as u64,
            digest: membership.finish().digest(),
        },
    ))
}

fn frame_format(frame: &DurableFrame<'_>) -> [u8; 10] {
    frame.format
}

fn reference_end(flag_offset: usize) -> usize {
    match flag_offset {
        40 => 120,
        120 => 152,
        160 => 224,
        232 => 296,
        296 => 320,
        _ => flag_offset,
    }
}
