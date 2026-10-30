pub(crate) use crate::history::parent_oracle::evidence_digest::digest_bytes;
pub(crate) use {
    crate::history::parent_oracle::evidence_digest::DigestBuilder,
    crate::history::parent_oracle::evidence_digest::DigestObservation,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::empty_facts,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::read_u16,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::read_u32,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::read_u64,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::residue,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::ArtifactFacts,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::ManifestFacts,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::PageFacts,
    crate::phase_eight_process::history::parent_oracle::artifact_facts::SelectorFacts,
};

pub(crate) const DURABLE_HEADER_BYTES: usize = 48;
pub(crate) const ROOT_MANIFEST_KIND: u8 = 2;
pub(crate) const SEGMENT_MANIFEST_KIND: u8 = 5;
pub(crate) const EXTENT_MANIFEST_KIND: u8 = 6;
pub(crate) const FREE_SPACE_MANIFEST_KIND: u8 = 7;
pub(crate) const ROOT_ROUTING_KIND: u8 = 8;
pub(crate) const SEGMENT_MEMBERSHIP_KIND: u8 = 9;
pub(crate) const FREE_SPACE_MEMBERSHIP_KIND: u8 = 10;
pub(crate) const ROOT_SELECTOR_KIND: u8 = 11;

#[derive(Clone, Copy)]
pub(crate) struct RawFrame<'bytes> {
    pub(crate) kind: u8,
    pub(crate) format: [u8; 10],
    pub(crate) identity: u64,
    pub(crate) page_lsn: u64,
    pub(crate) payload: &'bytes [u8],
}

pub(crate) fn observe(bytes: &[u8]) -> ArtifactFacts {
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

pub(crate) fn decode(bytes: &[u8]) -> Option<RawFrame<'_>> {
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
    if read_u16(&format, 0)? != 2
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

pub(crate) struct Semantic {
    pub(crate) generation: bool,
    pub(crate) generation_links: DigestObservation,
    pub(crate) selector: Option<SelectorFacts>,
    pub(crate) manifest: Option<ManifestFacts>,
}

pub(crate) fn page_digest(frame: &RawFrame<'_>) -> DigestObservation {
    let mut digest = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.page-lsn.v1",
    );
    let mut record = Vec::with_capacity(17);
    record.push(frame.kind);
    record.extend_from_slice(&frame.identity.to_le_bytes());
    record.extend_from_slice(&frame.page_lsn.to_le_bytes());
    digest.record(&record);
    digest.finish()
}

pub(crate) fn semantic(frame: &RawFrame<'_>) -> Option<Semantic> {
    match frame.kind {
        ROOT_SELECTOR_KIND => selector(frame),
        ROOT_MANIFEST_KIND => root_manifest(frame),
        SEGMENT_MANIFEST_KIND => counted_manifest(frame, 24, 40),
        EXTENT_MANIFEST_KIND => extent_manifest(frame),
        FREE_SPACE_MANIFEST_KIND => free_space_manifest(frame),
        ROOT_ROUTING_KIND => routing_manifest(frame, 88, 72),
        SEGMENT_MEMBERSHIP_KIND => routing_manifest(frame, 40, 56),
        FREE_SPACE_MEMBERSHIP_KIND => routing_manifest(frame, 40, 72),
        _ => None,
    }
}

pub(crate) fn selector(frame: &RawFrame<'_>) -> Option<Semantic> {
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
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
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

pub(crate) fn root_manifest(frame: &RawFrame<'_>) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() != 336
        || frame.identity == 0
        || read_u64(payload, 0)? != frame.identity
        || payload[18..24] != [0; 6]
        || payload[41..48] != [0; 7]
        || payload[121..128] != [0; 7]
        || payload[156..160] != [0; 4]
        || payload[161..168] != [0; 7]
        || payload[233..240] != [0; 7]
        || payload[313..320] != [0; 7]
        || [40, 120, 160, 232, 312]
            .iter()
            .any(|offset| payload[*offset] > 1)
    {
        return None;
    }
    let members = [40, 120, 160, 232, 312]
        .into_iter()
        .filter(|offset| payload[*offset] == 1)
        .count() as u64;
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
    generation.record(&payload[..40]);
    for offset in [40, 120, 160, 232, 312] {
        if payload[offset] == 1 {
            generation.record(&payload[offset..reference_end(offset)]);
        }
    }
    let mut membership = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.manifest-membership.v1",
    );
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

pub(crate) fn counted_manifest(
    frame: &RawFrame<'_>,
    header: usize,
    member_width: usize,
) -> Option<Semantic> {
    let count = usize::try_from(read_u32(frame.payload, 12)?).ok()?;
    if frame.payload.len() < header
        || count == 0
        || frame.payload.len() != header.checked_add(count.checked_mul(member_width)?)?
    {
        return None;
    }
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
    generation.record(&frame.payload[..16]);
    let mut membership = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.manifest-membership.v1",
    );
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

pub(crate) fn extent_manifest(frame: &RawFrame<'_>) -> Option<Semantic> {
    let alignment = read_u64(frame.payload, 48)?;
    if frame.payload.len() != 56 || !alignment.is_power_of_two() || frame.identity == 0 {
        return None;
    }
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
    generation.record(&frame.payload[..48]);
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

pub(crate) fn free_space_manifest(frame: &RawFrame<'_>) -> Option<Semantic> {
    let payload = frame.payload;
    if payload.len() != 168
        || frame.identity == 0
        || read_u64(payload, 0)? != frame.identity
        || payload[22..24] != [0; 2]
        || payload[65..72] != [0; 7]
        || payload[64] > 1
        || read_u64(payload, 144)? == 0
        || read_u64(payload, 152)? == 0
        || !read_u64(payload, 160)?.is_power_of_two()
        || read_u64(payload, 152)? % read_u64(payload, 160)? != 0
    {
        return None;
    }
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
    generation.record(&payload[..64]);
    let mut membership = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.manifest-membership.v1",
    );
    membership.record(payload);
    Some(Semantic {
        generation: true,
        generation_links: generation.finish(),
        selector: None,
        manifest: Some(ManifestFacts {
            count: 1,
            members: read_u64(payload, 24)?.checked_add(u64::from(payload[64]))?,
            digest: membership.finish().digest(),
        }),
    })
}

pub(crate) fn routing_manifest(
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
    let mut generation = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.generation-link.v1",
    );
    generation.record(&payload[..32]);
    let mut membership = crate::history::parent_oracle::evidence_digest::DigestBuilder::new(
        b"worth.store.recovery-observer.manifest-membership.v1",
    );
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

pub(crate) fn reference_end(offset: usize) -> usize {
    match offset {
        40 => 120,
        120 => 152,
        160 => 224,
        232 => 312,
        312 => 336,
        _ => offset,
    }
}

pub(crate) fn crc32c(bytes: &[u8]) -> u32 {
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
