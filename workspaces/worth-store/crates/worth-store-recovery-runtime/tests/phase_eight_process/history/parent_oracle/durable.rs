use super::evidence_digest::digest_bytes;
use super::{
    empty_facts, read_u16, read_u32, read_u64, residue, ArtifactFacts, DigestBuilder,
    DigestObservation, ManifestFacts, PageFacts, SelectorFacts,
};

const DURABLE_HEADER_BYTES: usize = 48;
const ROOT_MANIFEST_KIND: u8 = 2;
const SEGMENT_MANIFEST_KIND: u8 = 5;
const EXTENT_MANIFEST_KIND: u8 = 6;
const FREE_SPACE_MANIFEST_KIND: u8 = 7;
const ROOT_ROUTING_KIND: u8 = 8;
const SEGMENT_MEMBERSHIP_KIND: u8 = 9;
const FREE_SPACE_MEMBERSHIP_KIND: u8 = 10;
const ROOT_SELECTOR_KIND: u8 = 11;

#[derive(Clone, Copy)]
struct RawFrame<'bytes> {
    kind: u8,
    format: [u8; 10],
    identity: u64,
    page_lsn: u64,
    payload: &'bytes [u8],
}

pub(super) fn observe(bytes: &[u8]) -> ArtifactFacts {
    let Some(frame) = decode(bytes) else {
        return ArtifactFacts {
            residue: Some(residue(bytes)),
            ..empty_facts()
        };
    };
    let page = PageFacts {
        count: 1,
        minimum: Some(frame.page_lsn),
        maximum: Some(frame.page_lsn),
        digest: page_digest(&frame),
    };
    let Some(semantic) = semantic(&frame) else {
        return ArtifactFacts {
            page: Some(page),
            residue: Some(residue(bytes)),
            ..empty_facts()
        };
    };
    ArtifactFacts {
        generation: semantic.generation,
        generation_links: semantic.generation_links,
        selector: semantic.selector,
        page: Some(page),
        manifest: semantic.manifest,
        ..empty_facts()
    }
}

fn decode(bytes: &[u8]) -> Option<RawFrame<'_>> {
    if bytes.len() < DURABLE_HEADER_BYTES
        || bytes.get(..8) != Some(b"WRC5FRM\0")
        || !matches!(bytes[8], 1..=11)
        || bytes[9] != 2
        || read_u16(bytes, 20)? as usize != DURABLE_HEADER_BYTES
        || bytes[22..24] != [0; 2]
    {
        return None;
    }
    let format: [u8; 10] = bytes[10..20].try_into().ok()?;
    if read_u16(&format, 0)? != 1
        || !matches!(read_u32(&format, 2)?, 16_384 | 32_768 | 65_536)
        || format[6..10] != [1, 1, 1, 24]
    {
        return None;
    }
    let payload_bytes = usize::try_from(read_u32(bytes, 24)?).ok()?;
    let total = DURABLE_HEADER_BYTES.checked_add(payload_bytes)?;
    if bytes.len() != total {
        return None;
    }
    let mut covered = Vec::with_capacity(44 + payload_bytes);
    covered.extend_from_slice(&bytes[..44]);
    covered.extend_from_slice(&bytes[DURABLE_HEADER_BYTES..]);
    if crc32c(&covered) != read_u32(bytes, 44)? {
        return None;
    }
    Some(RawFrame {
        kind: bytes[8],
        format,
        identity: read_u64(bytes, 28)?,
        page_lsn: read_u64(bytes, 36)?,
        payload: &bytes[DURABLE_HEADER_BYTES..],
    })
}

struct Semantic {
    generation: bool,
    generation_links: DigestObservation,
    selector: Option<SelectorFacts>,
    manifest: Option<ManifestFacts>,
}

fn page_digest(frame: &RawFrame<'_>) -> DigestObservation {
    let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.page-lsn.v1");
    let mut record = Vec::with_capacity(17);
    record.push(frame.kind);
    record.extend_from_slice(&frame.identity.to_le_bytes());
    record.extend_from_slice(&frame.page_lsn.to_le_bytes());
    digest.record(&record);
    digest.finish()
}

fn semantic(frame: &RawFrame<'_>) -> Option<Semantic> {
    match frame.kind {
        ROOT_SELECTOR_KIND => selector(frame),
        ROOT_MANIFEST_KIND => root_manifest(frame),
        SEGMENT_MANIFEST_KIND => counted_manifest(frame, 24, 40),
        EXTENT_MANIFEST_KIND => fixed_manifest(frame, 56, 48),
        FREE_SPACE_MANIFEST_KIND => free_space_manifest(frame),
        ROOT_ROUTING_KIND => routing_manifest(frame, 88, 72),
        SEGMENT_MEMBERSHIP_KIND | FREE_SPACE_MEMBERSHIP_KIND => routing_manifest(frame, 40, 56),
        _ => None,
    }
}

fn selector(frame: &RawFrame<'_>) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() != 59
        || frame.identity == 0
        || payload[..16] == [0; 16]
        || !matches!(payload[16], 1 | 2)
        || payload[17..25] == [0; 8]
        || payload[51..] != [0; 8]
        || payload[41..51] != frame.format
    {
        return None;
    }
    let linked = read_u64(payload, 25)?;
    let linked_generation = read_u64(payload, 33)?;
    if (linked == 0) != (linked_generation == 0) {
        return None;
    }
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    let mut record = Vec::with_capacity(33);
    record.extend_from_slice(&frame.identity.to_le_bytes());
    record.extend_from_slice(&read_u64(payload, 17)?.to_le_bytes());
    record.extend_from_slice(&linked.to_le_bytes());
    record.extend_from_slice(&linked_generation.to_le_bytes());
    generation.record(&record);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: Some(SelectorFacts {
            identity: frame.identity,
            linked: (linked != 0).then_some(linked),
            store: payload[..16].try_into().ok()?,
            role: payload[16],
            generation: read_u64(payload, 17)?,
        }),
        manifest: None,
    })
}

fn root_manifest(frame: &RawFrame<'_>) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() != 320
        || frame.identity == 0
        || read_u64(payload, 0)? != frame.identity
        || payload[18..24] != [0; 6]
        || payload[41..48] != [0; 7]
        || payload[121..128] != [0; 7]
        || payload[156..160] != [0; 4]
        || payload[161..168] != [0; 7]
        || payload[233..240] != [0; 7]
        || payload[297..304] != [0; 7]
        || [40, 120, 160, 232, 296]
            .iter()
            .any(|offset| payload[*offset] > 1)
    {
        return None;
    }
    let members = [40, 120, 160, 232, 296]
        .into_iter()
        .filter(|offset| payload[*offset] == 1)
        .count() as u64;
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&payload[..40]);
    for offset in [40, 120, 160, 232, 296] {
        if payload[offset] == 1 {
            generation.record(&payload[offset..reference_end(offset)]);
        }
    }
    let mut membership =
        DigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(payload);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members,
            digest: membership.finish().digest(),
        }),
    })
}

fn counted_manifest(frame: &RawFrame<'_>, header: usize, member_width: usize) -> Option<Semantic> {
    let count = usize::try_from(read_u32(frame.payload, 12)?).ok()?;
    if frame.payload.len() < header
        || count == 0
        || frame.payload.len() != header.checked_add(count.checked_mul(member_width)?)?
    {
        return None;
    }
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..16]);
    let mut membership =
        DigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(frame.payload);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members: count as u64,
            digest: membership.finish().digest(),
        }),
    })
}

fn fixed_manifest(frame: &RawFrame<'_>, length: usize, generation_end: usize) -> Option<Semantic> {
    if frame.payload.len() != length
        || frame.payload[generation_end..] != [0; 8]
        || frame.identity == 0
    {
        return None;
    }
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&frame.payload[..generation_end]);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members: 1,
            digest: digest_bytes(frame.payload),
        }),
    })
}

fn free_space_manifest(frame: &RawFrame<'_>) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() != 128
        || frame.identity == 0
        || read_u64(payload, 0)? != frame.identity
        || payload[22..24] != [0; 2]
        || payload[65..72] != [0; 7]
        || payload[64] > 1
    {
        return None;
    }
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&payload[..64]);
    let mut membership =
        DigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(payload);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members: read_u64(payload, 24)? + u64::from(payload[64]),
            digest: membership.finish().digest(),
        }),
    })
}

fn routing_manifest(
    frame: &RawFrame<'_>,
    leaf_width: usize,
    branch_width: usize,
) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() < 40
        || frame.identity == 0
        || payload[21..24] != [0; 3]
        || payload[32..40] != [0; 8]
        || read_u64(payload, 0)? == 0
        || read_u64(payload, 8)? != frame.identity
        || read_u64(payload, 24)? == 0
    {
        return None;
    }
    let count = usize::from(read_u16(payload, 18)?);
    let width = match (payload[20], read_u16(payload, 16)?) {
        (1, 0) => leaf_width,
        (2, level) if level != 0 => branch_width,
        _ => return None,
    };
    if count == 0 || payload.len() != 40usize.checked_add(count.checked_mul(width)?)? {
        return None;
    }
    let mut generation = DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    generation.record(&payload[..32]);
    let mut membership =
        DigestBuilder::new(b"worth.store.recovery-observer.manifest-membership.v1");
    membership.record(payload);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members: count as u64,
            digest: membership.finish().digest(),
        }),
    })
}

fn reference_end(offset: usize) -> usize {
    match offset {
        40 => 120,
        120 => 152,
        160 => 224,
        232 => 296,
        296 => 320,
        _ => offset,
    }
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut value = !0_u32;
    for byte in bytes {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}
