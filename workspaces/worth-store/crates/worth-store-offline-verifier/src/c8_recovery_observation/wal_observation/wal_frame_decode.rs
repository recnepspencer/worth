use super::super::physical_format;
use super::super::RecoveryObserverWalTopologyDenial;

const HEADER_BYTES: usize = 116;
const FOOTER_BYTES: usize = 32;

pub(super) struct DecodedWalFrame {
    pub(super) segment_id: u64,
    pub(super) generation: u64,
    pub(super) lsn_start: u64,
    pub(super) lsn_end: u64,
    pub(super) total_bytes: usize,
}

pub(super) enum FrameDecode {
    Valid(DecodedWalFrame),
    Stop(
        Option<
            super::super::observer_evidence_accumulation::RecoveryObserverWalTopologyObservation,
        >,
    ),
}

pub(super) fn decode(
    bytes: &[u8],
    offset: usize,
    expected_segment: Option<u64>,
    expected_generation: Option<u64>,
    previous_lsn_end: Option<u64>,
) -> FrameDecode {
    let Some(header) = bytes.get(offset..offset.saturating_add(HEADER_BYTES)) else {
        return FrameDecode::Stop(None);
    };
    if header.len() != HEADER_BYTES {
        return FrameDecode::Stop(None);
    }
    let valid_header = header.get(..8) == Some(&physical_format::WAL_MAGIC[..])
        && physical_format::read_u16(header, 8) == Some(1)
        && physical_format::read_u16(header, 10) == Some(HEADER_BYTES as u16);
    if !valid_header {
        let topology = (offset == 0 && header.get(..8) == Some(&physical_format::WAL_MAGIC[..]))
            .then(|| topology_denial(header, RecoveryObserverWalTopologyDenial::MalformedFrame));
        return FrameDecode::Stop(topology);
    }
    let Some(payload_bytes) =
        physical_format::read_u64(header, 44).and_then(|value| usize::try_from(value).ok())
    else {
        return FrameDecode::Stop(None);
    };
    let Some(total_bytes) = HEADER_BYTES
        .checked_add(payload_bytes)
        .and_then(|value| value.checked_add(FOOTER_BYTES))
    else {
        return FrameDecode::Stop(None);
    };
    let Some(frame) = bytes.get(offset..offset.saturating_add(total_bytes)) else {
        return FrameDecode::Stop(None);
    };
    if frame.len() != total_bytes {
        return FrameDecode::Stop(None);
    }
    let Some(segment_id) = physical_format::read_u64(header, 12) else {
        return FrameDecode::Stop(None);
    };
    let Some(generation) = physical_format::read_u64(header, 20) else {
        return FrameDecode::Stop(None);
    };
    let Some(lsn_start) = physical_format::read_u64(header, 28) else {
        return FrameDecode::Stop(None);
    };
    let Some(lsn_end) = physical_format::read_u64(header, 36) else {
        return FrameDecode::Stop(None);
    };
    let topology =
        if segment_id == 0 || generation == 0 || lsn_start >= lsn_end || payload_bytes == 0 {
            Some(topology_denial(
                header,
                RecoveryObserverWalTopologyDenial::MalformedFrame,
            ))
        } else if expected_segment.is_some_and(|expected| expected != segment_id) {
            Some(topology_denial(
                header,
                RecoveryObserverWalTopologyDenial::SegmentIdentityMismatch,
            ))
        } else if expected_generation.is_some_and(|expected| expected != generation) {
            Some(topology_denial(
                header,
                RecoveryObserverWalTopologyDenial::GenerationMismatch,
            ))
        } else if previous_lsn_end.is_some_and(|previous| previous != lsn_start) {
            Some(topology_denial(
                header,
                RecoveryObserverWalTopologyDenial::NonContiguousLsn,
            ))
        } else {
            None
        };
    if let Some(topology) = topology {
        return FrameDecode::Stop(Some(topology));
    }
    let payload = &frame[HEADER_BYTES..HEADER_BYTES + payload_bytes];
    let payload_digest = physical_format::digest_bytes(payload);
    let frame_digest = physical_format::digest_bytes(&frame[..HEADER_BYTES + payload_bytes]);
    if header[84..116] != payload_digest || frame[HEADER_BYTES + payload_bytes..] != frame_digest {
        return FrameDecode::Stop(None);
    }
    FrameDecode::Valid(DecodedWalFrame {
        segment_id,
        generation,
        lsn_start,
        lsn_end,
        total_bytes,
    })
}

fn topology_denial(
    header: &[u8],
    denial: RecoveryObserverWalTopologyDenial,
) -> super::super::observer_evidence_accumulation::RecoveryObserverWalTopologyObservation {
    super::super::observer_evidence_accumulation::RecoveryObserverWalTopologyObservation {
        segment: physical_format::read_u64(header, 12).unwrap_or(0),
        generation: physical_format::read_u64(header, 20).unwrap_or(0),
        first_lsn: physical_format::read_u64(header, 28).unwrap_or(0),
        last_lsn: physical_format::read_u64(header, 36).unwrap_or(0),
        denial: Some(denial),
    }
}
