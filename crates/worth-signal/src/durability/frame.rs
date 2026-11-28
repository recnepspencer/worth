//! Versioned Signal owner envelopes, independent of snapshots and derived state.
use super::SignalAuthoredImageDenial;
use sha2::{Digest, Sha256};
use worth_execution::{
    charge_framed_bytes, encode_framed_array, inspect_framed_envelope, write_framed_envelope,
    ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder, FramedEnvelopeError,
    FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity, BoundaryProtocolVersion,
};
mod fields;
pub(super) use fields::{AssociationFields, Reader, Writer};
pub const SIGNAL_OWNER_FRAME_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.signal.owner-frame");
pub const SIGNAL_OWNER_FRAME_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(2);
pub(super) const HEADER: usize = FRAMED_ENVELOPE_HEADER_BYTES + 4;
pub const SIGNAL_OWNER_FRAME_MAXIMUM_BYTES: usize = 96 << 20;
fn spec(maximum: usize) -> FramedEnvelopeSpec {
    FramedEnvelopeSpec {
        magic: *b"WSIGOWNR",
        supported_versions: const { &[SIGNAL_OWNER_FRAME_VERSION.get()] },
        max_payload_bytes: maximum.saturating_sub(FRAMED_ENVELOPE_HEADER_BYTES) as u64,
    }
}

/// Bind transient owner meaning with the same field grammar, without an artifact.
pub(super) fn commitment(
    execution: &ExecutionRequest<'_, '_>,
    domain: &[u8],
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), SignalDurableFrameDenial>,
) -> Result<[u8; 32], SignalDurableFrameDenial> {
    let mut size = 0usize;
    emit(&mut Writer::new(&mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= SIGNAL_OWNER_FRAME_MAXIMUM_BYTES)
            .ok_or(SignalDurableFrameDenial::SizeLimit)?;
        Ok(())
    }))?;
    admit_work(execution, size.saturating_mul(2))?;
    let mut hash = Sha256::new();
    hash.update(domain);
    // Transient commitments retain their original field grammar and version.
    hash.update(1u32.to_le_bytes());
    emit(&mut Writer::new(&mut |bytes| {
        hash.update(bytes);
        Ok(())
    }))?;
    Ok(hash.finalize().into())
}

#[derive(Debug)]
pub enum SignalDurableFrameDenial {
    Malformed,
    SizeLimit,
    ChecksumMismatch,
    ForeignOwner,
    ForeignBranch,
    SnapshotRequired,
    UnsupportedVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    Image(SignalAuthoredImageDenial),
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
}
pub(super) fn encode(
    kind: u32,
    execution: &ExecutionRequest<'_, '_>,
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), SignalDurableFrameDenial>,
) -> Result<ExecutionImmutableBytes, SignalDurableFrameDenial> {
    encode_bounded(kind, execution, SIGNAL_OWNER_FRAME_MAXIMUM_BYTES, emit)
}
pub(super) fn encode_bounded(
    kind: u32,
    execution: &ExecutionRequest<'_, '_>,
    maximum: usize,
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), SignalDurableFrameDenial>,
) -> Result<ExecutionImmutableBytes, SignalDurableFrameDenial> {
    encode_framed_array(
        spec(maximum),
        SIGNAL_OWNER_FRAME_VERSION.get(),
        execution,
        |sink| {
            sink(&kind.to_le_bytes())?;
            emit(&mut Writer::new(sink))
        },
        SignalDurableFrameDenial::Backing,
    )
    .map(|array| array.into_immutable_bytes())
}

/// Emit the frame into another owner's admitted transport without another buffer.
pub(super) fn write(
    kind: u32,
    execution: &ExecutionRequest<'_, '_>,
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), SignalDurableFrameDenial>,
    sink: &mut dyn FnMut(&[u8]) -> Result<(), SignalDurableFrameDenial>,
) -> Result<(), SignalDurableFrameDenial> {
    write_framed_envelope(
        spec(SIGNAL_OWNER_FRAME_MAXIMUM_BYTES),
        SIGNAL_OWNER_FRAME_VERSION.get(),
        execution,
        |sink| {
            sink(&kind.to_le_bytes())?;
            emit(&mut Writer::new(sink))
        },
        sink,
    )
}

pub(super) fn payload(bytes: &[u8], kind: u32) -> Result<Reader<'_>, SignalDurableFrameDenial> {
    payload_bounded(bytes, kind, SIGNAL_OWNER_FRAME_MAXIMUM_BYTES)
}

pub(super) fn payload_bounded(
    bytes: &[u8],
    kind: u32,
    maximum: usize,
) -> Result<Reader<'_>, SignalDurableFrameDenial> {
    use SignalDurableFrameDenial as D;
    if bytes.len() > maximum {
        return Err(D::SizeLimit);
    }
    let (_, payload) = inspect_framed_envelope(spec(maximum), bytes, |_| Ok::<_, D>(()))?;
    let (recorded, payload) = payload.split_at_checked(4).ok_or(D::Malformed)?;
    if recorded != kind.to_le_bytes() {
        return Err(D::Malformed);
    }
    Ok(Reader::new(payload))
}

impl From<FramedEnvelopeError> for SignalDurableFrameDenial {
    fn from(error: FramedEnvelopeError) -> Self {
        use FramedEnvelopeError as Error;
        match error {
            Error::Execution(cause) => Self::Request(cause),
            Error::ChecksumMismatch => Self::ChecksumMismatch,
            Error::PayloadTooLarge { .. } | Error::Allocation => Self::SizeLimit,
            Error::UnsupportedVersion { found } => BoundaryProtocolVersion::try_new(found)
                .ok()
                .and_then(|found| {
                    BoundaryProtocolCompatibilityWindow::inclusive(
                        SIGNAL_OWNER_FRAME_VERSION,
                        SIGNAL_OWNER_FRAME_VERSION,
                    )
                    .admit(found)
                    .err()
                })
                .map_or(Self::Malformed, Self::UnsupportedVersion),
            _ => Self::Malformed,
        }
    }
}
pub(super) fn copy_bytes(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<ExecutionImmutableBytes, SignalDurableFrameDenial> {
    let mut out = ExecutionRequestArrayBuilder::allocate(*execution, bytes.len())
        .map_err(SignalDurableFrameDenial::Backing)?;
    out.extend_from_slice(bytes)
        .map_err(SignalDurableFrameDenial::Backing)?;
    Ok(out
        .seal()
        .map_err(SignalDurableFrameDenial::Backing)?
        .into_immutable_bytes())
}
pub(super) fn admit_work(
    execution: &ExecutionRequest<'_, '_>,
    size: usize,
) -> Result<(), SignalDurableFrameDenial> {
    admit_work_bounded(execution, size, SIGNAL_OWNER_FRAME_MAXIMUM_BYTES)
}
pub(super) fn admit_work_bounded(
    execution: &ExecutionRequest<'_, '_>,
    size: usize,
    maximum: usize,
) -> Result<(), SignalDurableFrameDenial> {
    if size > maximum.saturating_mul(3) {
        return Err(SignalDurableFrameDenial::SizeLimit);
    }
    charge_framed_bytes(execution, size).map_err(SignalDurableFrameDenial::Request)
}
