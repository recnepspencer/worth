use super::*;
use crate::{ExtentArenaId, PersistedRecordIdentity, PhysicalExtentId};

impl PhysicalExtentCopyRecord {
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = EXTENT_COPY_DOMAIN.to_vec();
        match self {
            Self::Intent(intent) => {
                bytes.push(1);
                bytes.extend_from_slice(&intent.operation);
                push_u64(&mut bytes, intent.source_root);
                bytes.extend_from_slice(&intent.source.record().allocation_epoch());
                push_u64(&mut bytes, intent.source.record().ordinal());
                push_u64(&mut bytes, 0);
                for value in [
                    intent.source.extent().get(),
                    intent.source.extent_generation(),
                    intent.destination.extent_generation(),
                ] {
                    push_u64(&mut bytes, value);
                }
                for range in [
                    intent.source.arena_range(),
                    intent.destination.arena_range(),
                ] {
                    for value in [range.arena().get(), range.offset(), range.length()] {
                        push_u64(&mut bytes, value);
                    }
                }
                push_u64(&mut bytes, intent.source.payload_bytes());
                push_u64(&mut bytes, intent.alignment);
                bytes.extend_from_slice(&intent.maximum_frame_bytes.to_le_bytes());
                bytes.extend_from_slice(&intent.chunk_count.to_le_bytes());
                bytes.extend_from_slice(&intent.source_digest);
            }
            Self::Resolved(resolution) => {
                bytes.push(2);
                bytes.extend_from_slice(&resolution.operation);
                bytes.extend_from_slice(&resolution.intent_digest);
                push_u64(&mut bytes, resolution.intent_lsn);
                let (tag, root, publication) = match resolution.kind {
                    PhysicalExtentCopyResolutionKind::Cancelled => (0, 0, 0),
                    PhysicalExtentCopyResolutionKind::Published {
                        root_generation,
                        publication_lsn,
                    } => (1, root_generation, publication_lsn),
                };
                bytes.push(tag);
                push_u64(&mut bytes, root);
                push_u64(&mut bytes, publication);
            }
        }
        bytes
    }

    pub fn decode(
        bytes: &[u8],
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<Self, PhysicalExtentCopyDenial> {
        let body = bytes
            .strip_prefix(EXTENT_COPY_DOMAIN)
            .ok_or(PhysicalExtentCopyDenial::Domain)?;
        match body.split_first() {
            Some((1, body)) if body.len() == 200 => decode_intent(body, format).map(Self::Intent),
            Some((2, body)) if body.len() == 89 => decode_resolution(body).map(Self::Resolved),
            Some((1 | 2, _)) => Err(PhysicalExtentCopyDenial::Length),
            _ => Err(PhysicalExtentCopyDenial::Tag),
        }
    }
}

fn decode_intent(
    body: &[u8],
    format: PhysicalRecordFormatDeclaration,
) -> Result<PhysicalExtentCopyIntent, PhysicalExtentCopyDenial> {
    use PhysicalExtentCopyDenial::{Geometry, Identity};
    let operation = body[..32].try_into().unwrap();
    if read_u64(body, 64) != 0 {
        return Err(Identity);
    }
    let record = PersistedRecordIdentity::new(body[40..56].try_into().unwrap(), read_u64(body, 56))
        .ok_or(Identity)?;
    let extent = PhysicalExtentId::from_raw(read_u64(body, 72)).map_err(|_| Identity)?;
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent)
        .with_extent_generation(
            PhysicalGeneration::from_raw(read_u64(body, 80)).map_err(|_| Identity)?,
        );
    let range = |start| {
        ExtentArenaRange::new(
            ExtentArenaId::new(read_u64(body, start))?,
            read_u64(body, start + 8),
            read_u64(body, start + 16),
        )
    };
    let source = DurableExtentRecordPlacement::new(
        record,
        cell,
        read_u64(body, 144),
        range(96).ok_or(Geometry)?,
    )
    .ok_or(Identity)?;
    let intent = PhysicalExtentCopyIntent::new(
        format,
        operation,
        read_u64(body, 32),
        source,
        range(120).ok_or(Geometry)?,
        read_u64(body, 152),
        body[168..200].try_into().unwrap(),
    )
    .ok_or(Geometry)?;
    if intent.destination.extent_generation() != read_u64(body, 88)
        || intent.maximum_frame_bytes != u32::from_le_bytes(body[160..164].try_into().unwrap())
        || intent.chunk_count != u32::from_le_bytes(body[164..168].try_into().unwrap())
    {
        return Err(Geometry);
    }
    Ok(intent)
}

fn decode_resolution(
    body: &[u8],
) -> Result<PhysicalExtentCopyResolution, PhysicalExtentCopyDenial> {
    let root = read_u64(body, 73);
    let publication = read_u64(body, 81);
    let kind = match body[72] {
        0 if root == 0 && publication == 0 => PhysicalExtentCopyResolutionKind::Cancelled,
        1 => PhysicalExtentCopyResolutionKind::Published {
            root_generation: root,
            publication_lsn: publication,
        },
        _ => return Err(PhysicalExtentCopyDenial::Tag),
    };
    PhysicalExtentCopyResolution::new(
        body[..32].try_into().unwrap(),
        body[32..64].try_into().unwrap(),
        read_u64(body, 64),
        kind,
    )
    .ok_or(PhysicalExtentCopyDenial::Identity)
}

fn read_u64(bytes: &[u8], start: usize) -> u64 {
    u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap())
}
fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
