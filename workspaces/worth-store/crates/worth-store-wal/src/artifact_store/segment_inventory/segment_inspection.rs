use sha2::{Digest, Sha256};
use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, WalFrameV1ChecksumCalculator, WAL_FRAME_V1_FOOTER_BYTES,
    WAL_FRAME_V1_HEADER_BYTES,
};

use super::WalSegmentArtifactIdentity;
use crate::{LogSequenceNumber, WalArtifactStoreDenial, WalLsnRange};

mod accessors;
mod denial;
mod owned_artifact;
mod owned_frame;

pub use denial::{WalActiveTailInspectionDenial, WalActiveTailInspectionFailure};
pub use owned_artifact::VerifiedWalArtifact;
pub use owned_frame::VerifiedWalFrame;

/// Complete, digest-verified facts reconstructed from one bounded WAL segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalSegmentInspection {
    identity: WalSegmentArtifactIdentity,
    lsn_range: WalLsnRange,
    frame_count: u64,
    byte_count: u64,
    artifact_digest: [u8; 32],
}

/// One frame payload borrowed from the exact complete segment bytes whose
/// integrity and topology were admitted by [`inspect_verified_wal_segment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedWalFramePayload<'segment> {
    lsn_range: WalLsnRange,
    payload: &'segment [u8],
    encoded_bytes: u64,
}

/// Complete segment facts plus payload views from the same verification pass.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedWalSegment<'segment> {
    inspection: WalSegmentInspection,
    frames: Vec<VerifiedWalFramePayload<'segment>>,
}

/// Verified complete frames from the active WAL artifact plus proof that any
/// remaining bytes are only an incomplete final frame.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedWalActiveTail<'segment> {
    verified_prefix: VerifiedWalSegment<'segment>,
    interrupted_tail: Option<InterruptedWalTail>,
}

/// Store-unforgeable coordinates for removing an interrupted final frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptedWalTail {
    valid_prefix_bytes: u64,
    observed_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptedWalSegmentStart {
    observed_bytes: u64,
}

enum ActiveTailFrame<'segment> {
    Complete {
        lsn_start: u64,
        lsn_end: u64,
        payload: &'segment [u8],
        frame_end: usize,
    },
    Interrupted,
}

pub fn inspect_complete_wal_segment(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
) -> Result<WalSegmentInspection, WalArtifactStoreDenial> {
    Ok(inspect_verified_wal_segment(identity, bytes)?.inspection)
}

pub fn inspect_verified_wal_segment(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
) -> Result<VerifiedWalSegment<'_>, WalArtifactStoreDenial> {
    let active = inspect_verified_wal_active_tail(identity, bytes)?;
    if active.interrupted_tail.is_some() {
        return Err(WalArtifactStoreDenial::InvalidFrame);
    }
    Ok(active.verified_prefix)
}

pub fn inspect_verified_wal_active_tail(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
) -> Result<VerifiedWalActiveTail<'_>, WalArtifactStoreDenial> {
    inspect_wal_active_tail_with_evidence(identity, bytes).map_err(|failure| {
        match failure.denial() {
            WalActiveTailInspectionDenial::Artifact(denial) => denial,
            WalActiveTailInspectionDenial::FrameLimitExceeded { .. } => {
                unreachable!("the ordinary WAL inspector has no frame limit")
            }
        }
    })
}

fn inspect_wal_active_tail_with_evidence(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
) -> Result<VerifiedWalActiveTail<'_>, WalActiveTailInspectionFailure> {
    inspect_bounded_wal_active_tail_with_evidence(identity, bytes, u64::MAX)
}

pub fn inspect_bounded_wal_active_tail_with_evidence(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
    maximum_frames: u64,
) -> Result<VerifiedWalActiveTail<'_>, WalActiveTailInspectionFailure> {
    if bytes.is_empty() {
        return Err(inspection_failure(WalArtifactStoreDenial::InvalidFrame, 0));
    }
    let mut offset = 0usize;
    let mut first_lsn = None;
    let mut last_lsn_end = None;
    let mut frame_count = 0u64;
    let mut frames = Vec::new();
    while offset < bytes.len() {
        let inspected =
            inspect_active_tail_frame(identity, bytes, offset, last_lsn_end).map_err(|denial| {
                inspection_failure(
                    denial,
                    frame_count + u64::from(denial == WalArtifactStoreDenial::DigestMismatch),
                )
            })?;
        let ActiveTailFrame::Complete {
            lsn_start,
            lsn_end,
            payload,
            frame_end,
        } = inspected
        else {
            let observed = frame_count.saturating_add(1);
            if observed > maximum_frames {
                return Err(frame_limit_failure(observed, maximum_frames, observed));
            }
            return finish_interrupted_tail(
                identity,
                bytes,
                offset,
                first_lsn,
                last_lsn_end,
                frame_count,
                frames,
            )
            .map_err(|denial| inspection_failure(denial, frame_count));
        };
        let observed = frame_count.saturating_add(1);
        if observed > maximum_frames {
            return Err(frame_limit_failure(observed, maximum_frames, observed));
        }
        first_lsn.get_or_insert(lsn_start);
        last_lsn_end = Some(lsn_end);
        let lsn_range = WalLsnRange::new(
            LogSequenceNumber::new(lsn_start),
            LogSequenceNumber::new(lsn_end),
        )
        .map_err(|_| inspection_failure(WalArtifactStoreDenial::InvalidFrame, frame_count + 1))?;
        frames.push(VerifiedWalFramePayload {
            lsn_range,
            payload,
            encoded_bytes: (WAL_FRAME_V1_HEADER_BYTES + payload.len() + WAL_FRAME_V1_FOOTER_BYTES)
                as u64,
        });
        frame_count = frame_count.checked_add(1).ok_or_else(|| {
            inspection_failure(WalArtifactStoreDenial::InvalidFrame, frame_count + 1)
        })?;
        offset = frame_end;
    }
    let verified_prefix = finish_verified_segment(
        identity,
        bytes,
        first_lsn,
        last_lsn_end,
        frame_count,
        frames,
    )
    .map_err(|denial| inspection_failure(denial, frame_count))?;
    Ok(VerifiedWalActiveTail {
        verified_prefix,
        interrupted_tail: None,
    })
}

fn inspection_failure(
    denial: WalArtifactStoreDenial,
    frames_scanned: u64,
) -> WalActiveTailInspectionFailure {
    WalActiveTailInspectionFailure::artifact(denial, frames_scanned)
}

fn frame_limit_failure(
    observed: u64,
    admitted: u64,
    frames_scanned: u64,
) -> WalActiveTailInspectionFailure {
    debug_assert_eq!(observed, frames_scanned);
    WalActiveTailInspectionFailure::frame_limit(observed, admitted)
}

pub fn inspect_interrupted_wal_segment_start(
    identity: WalSegmentArtifactIdentity,
    bytes: &[u8],
) -> Result<InterruptedWalSegmentStart, WalArtifactStoreDenial> {
    if bytes.is_empty() {
        return Err(WalArtifactStoreDenial::InvalidFrame);
    }
    match inspect_active_tail_frame(identity, bytes, 0, None)? {
        ActiveTailFrame::Interrupted => Ok(InterruptedWalSegmentStart {
            observed_bytes: bytes.len() as u64,
        }),
        ActiveTailFrame::Complete { .. } => Err(WalArtifactStoreDenial::InvalidFrame),
    }
}

fn inspect_active_tail_frame<'segment>(
    identity: WalSegmentArtifactIdentity,
    bytes: &'segment [u8],
    offset: usize,
    last_lsn_end: Option<u64>,
) -> Result<ActiveTailFrame<'segment>, WalArtifactStoreDenial> {
    if bytes.len() - offset < WAL_FRAME_V1_HEADER_BYTES {
        return Ok(ActiveTailFrame::Interrupted);
    }
    let header_end = offset
        .checked_add(WAL_FRAME_V1_HEADER_BYTES)
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    let header: &[u8; WAL_FRAME_V1_HEADER_BYTES] = bytes
        .get(offset..header_end)
        .and_then(|header| header.try_into().ok())
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    let fields = decode_wal_frame_v1_header(header).map_err(super::super::map_wal_frame_denial)?;
    if fields.segment_id() != identity.segment().get()
        || fields.generation() != identity.generation().get()
    {
        return Err(WalArtifactStoreDenial::StoreBindingMismatch);
    }
    if last_lsn_end.is_some_and(|prior| prior != fields.lsn_start()) {
        return Err(WalArtifactStoreDenial::NonContiguousLsn);
    }
    let payload_bytes = usize::try_from(fields.payload_bytes())
        .map_err(|_| WalArtifactStoreDenial::InvalidFrame)?;
    let payload_end = header_end
        .checked_add(payload_bytes)
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    let frame_end = payload_end
        .checked_add(WAL_FRAME_V1_FOOTER_BYTES)
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    if frame_end > bytes.len() {
        return Ok(ActiveTailFrame::Interrupted);
    }
    let payload = &bytes[header_end..payload_end];
    let footer: &[u8; WAL_FRAME_V1_FOOTER_BYTES] = bytes[payload_end..frame_end]
        .try_into()
        .expect("fixed WAL footer");
    let mut checksums = WalFrameV1ChecksumCalculator::new(header);
    checksums
        .update_payload(payload)
        .map_err(super::super::map_wal_frame_denial)?;
    checksums
        .finish(fields, footer)
        .map_err(super::super::map_wal_frame_denial)?;
    Ok(ActiveTailFrame::Complete {
        lsn_start: fields.lsn_start(),
        lsn_end: fields.lsn_end(),
        payload,
        frame_end,
    })
}

fn finish_interrupted_tail<'segment>(
    identity: WalSegmentArtifactIdentity,
    bytes: &'segment [u8],
    valid_prefix_bytes: usize,
    first_lsn: Option<u64>,
    last_lsn_end: Option<u64>,
    frame_count: u64,
    frames: Vec<VerifiedWalFramePayload<'segment>>,
) -> Result<VerifiedWalActiveTail<'segment>, WalArtifactStoreDenial> {
    if valid_prefix_bytes == 0 {
        return Err(WalArtifactStoreDenial::InvalidFrame);
    }
    let verified_prefix = finish_verified_segment(
        identity,
        &bytes[..valid_prefix_bytes],
        first_lsn,
        last_lsn_end,
        frame_count,
        frames,
    )?;
    Ok(VerifiedWalActiveTail {
        verified_prefix,
        interrupted_tail: Some(InterruptedWalTail {
            valid_prefix_bytes: valid_prefix_bytes as u64,
            observed_bytes: bytes.len() as u64,
        }),
    })
}

fn finish_verified_segment<'segment>(
    identity: WalSegmentArtifactIdentity,
    bytes: &'segment [u8],
    first_lsn: Option<u64>,
    last_lsn_end: Option<u64>,
    frame_count: u64,
    frames: Vec<VerifiedWalFramePayload<'segment>>,
) -> Result<VerifiedWalSegment<'segment>, WalArtifactStoreDenial> {
    let start = LogSequenceNumber::new(first_lsn.ok_or(WalArtifactStoreDenial::InvalidFrame)?);
    let end = LogSequenceNumber::new(last_lsn_end.ok_or(WalArtifactStoreDenial::InvalidFrame)?);
    let lsn_range =
        WalLsnRange::new(start, end).map_err(|_| WalArtifactStoreDenial::InvalidFrame)?;
    Ok(VerifiedWalSegment {
        inspection: WalSegmentInspection {
            identity,
            lsn_range,
            frame_count,
            byte_count: bytes.len() as u64,
            artifact_digest: Sha256::digest(bytes).into(),
        },
        frames,
    })
}
