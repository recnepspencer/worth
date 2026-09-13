use sha2::{Digest, Sha256};

use super::super::evidence_digest::DigestBuilder;
use super::super::{read_u16, read_u64, residue, ArtifactFacts, WalFacts};

const WAL_HEADER_BYTES: usize = 116;
const WAL_FOOTER_BYTES: usize = 32;

pub(super) fn observe_wal(bytes: &[u8]) -> ArtifactFacts {
    let mut offset: usize = 0;
    let mut previous_end = None;
    let mut frames = 0;
    let mut first = None;
    let mut last = None;
    let mut segment = None;
    let mut generation = None;
    let mut wal_digest = DigestBuilder::new(b"worth.store.recovery-observer.wal-prefix.v1");
    let mut generation_digest =
        DigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1");
    while let Some(header) = bytes.get(offset..offset.saturating_add(WAL_HEADER_BYTES)) {
        if header.len() != WAL_HEADER_BYTES || header.get(..8) != Some(b"WORTHWAL") {
            break;
        }
        if read_u16(header, 8) != Some(1) || read_u16(header, 10) != Some(116) {
            break;
        }
        let Some(payload_bytes) =
            read_u64(header, 44).and_then(|value| usize::try_from(value).ok())
        else {
            break;
        };
        let Some(total) = WAL_HEADER_BYTES
            .checked_add(payload_bytes)
            .and_then(|value| value.checked_add(WAL_FOOTER_BYTES))
        else {
            break;
        };
        let Some(frame) = bytes.get(offset..offset.saturating_add(total)) else {
            break;
        };
        if frame.len() != total {
            break;
        }
        let (Some(frame_segment), Some(frame_generation), Some(start), Some(end)) = (
            read_u64(header, 12),
            read_u64(header, 20),
            read_u64(header, 28),
            read_u64(header, 36),
        ) else {
            break;
        };
        let payload = &frame[WAL_HEADER_BYTES..WAL_HEADER_BYTES + payload_bytes];
        let payload_digest: [u8; 32] = Sha256::digest(payload).into();
        let frame_digest: [u8; 32] =
            Sha256::digest(&frame[..WAL_HEADER_BYTES + payload_bytes]).into();
        if frame_segment == 0
            || frame_generation == 0
            || start >= end
            || payload_bytes == 0
            || header[84..116] != payload_digest
            || frame[WAL_HEADER_BYTES + payload_bytes..] != frame_digest
            || segment.is_some_and(|value| value != frame_segment)
            || generation.is_some_and(|value| value != frame_generation)
            || previous_end.is_some_and(|value| value != start)
        {
            break;
        }
        segment = Some(frame_segment);
        generation = Some(frame_generation);
        let mut record = Vec::with_capacity(32);
        record.extend_from_slice(&frame_segment.to_le_bytes());
        record.extend_from_slice(&frame_generation.to_le_bytes());
        record.extend_from_slice(&start.to_le_bytes());
        record.extend_from_slice(&end.to_le_bytes());
        wal_digest.record(&record);
        generation_digest.record(&record);
        first.get_or_insert(start);
        last = Some(end);
        previous_end = Some(end);
        frames += 1;
        offset += total;
    }
    let generation_links = generation_digest.finish();
    ArtifactFacts {
        generation: frames > 0,
        generation_links,
        wal: Some(WalFacts {
            segment,
            generation,
            valid_bytes: offset as u64,
            observed_bytes: bytes.len() as u64,
            frames,
            first,
            last,
            digest: wal_digest.finish().digest(),
        }),
        wal_residue: (offset < bytes.len()).then(|| residue(&bytes[offset..])),
        ..super::super::empty_facts()
    }
}
