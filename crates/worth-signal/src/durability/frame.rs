//! Versioned Signal owner envelopes, independent of snapshots and derived state.
use super::SignalAuthoredImageDenial;
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity, BoundaryProtocolVersion,
};
mod fields;
pub(super) use fields::{AssociationFields, Reader, Writer};
pub const SIGNAL_OWNER_FRAME_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.signal.owner-frame");
pub const SIGNAL_OWNER_FRAME_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
pub(super) const HEADER: usize = 56;
pub const SIGNAL_OWNER_FRAME_MAXIMUM_BYTES: usize = 96 << 20;
const MAGIC: &[u8; 8] = b"WSIGOWNR";

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
    hash.update(SIGNAL_OWNER_FRAME_VERSION.get().to_le_bytes());
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
    use SignalDurableFrameDenial as D;
    let mut size = 0usize;
    emit(&mut Writer::new(&mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|n| *n <= maximum.saturating_sub(HEADER))
            .ok_or(D::SizeLimit)?;
        Ok(())
    }))?;
    admit_work_bounded(execution, size.saturating_mul(3), maximum)?;
    let mut checksum = Sha256::new();
    let version = SIGNAL_OWNER_FRAME_VERSION.get().to_le_bytes();
    let kind = kind.to_le_bytes();
    let length = (size as u64).to_le_bytes();
    checksum.update(version);
    checksum.update(kind);
    checksum.update(length);
    emit(&mut Writer::new(&mut |bytes| {
        checksum.update(bytes);
        Ok(())
    }))?;
    let checksum: [u8; 32] = checksum.finalize().into();
    let mut out =
        ExecutionRequestArrayBuilder::allocate(*execution, HEADER + size).map_err(D::Backing)?;
    for field in [MAGIC.as_slice(), &version, &kind, &length, &checksum] {
        out.extend_from_slice(field).map_err(D::Backing)?;
    }
    emit(&mut Writer::new(&mut |bytes| {
        out.extend_from_slice(bytes).map_err(D::Backing)
    }))?;
    Ok(out.seal().map_err(D::Backing)?.into_immutable_bytes())
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
    if bytes.len() < HEADER || &bytes[..8] != MAGIC {
        return Err(D::Malformed);
    }
    let version =
        BoundaryProtocolVersion::try_new(u32::from_le_bytes(bytes[8..12].try_into().unwrap()))
            .map_err(|_| D::Malformed)?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        SIGNAL_OWNER_FRAME_VERSION,
        SIGNAL_OWNER_FRAME_VERSION,
    )
    .admit(version)
    .map_err(D::UnsupportedVersion)?;
    if u32::from_le_bytes(bytes[12..16].try_into().unwrap()) != kind
        || u64::from_le_bytes(bytes[16..24].try_into().unwrap()) != (bytes.len() - HEADER) as u64
    {
        return Err(D::Malformed);
    }
    let mut hash = Sha256::new();
    hash.update(&bytes[8..24]);
    hash.update(&bytes[HEADER..]);
    let digest: [u8; 32] = hash.finalize().into();
    if digest.as_slice() != &bytes[24..56] {
        return Err(D::ChecksumMismatch);
    }
    Ok(Reader::new(&bytes[HEADER..]))
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
    execution
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(size as u64)
            } else {
                context.poll()
            }
        })
        .map_err(SignalDurableFrameDenial::Request)?
        .0
        .map_err(|cause| {
            SignalDurableFrameDenial::Request(worth_execution::WorkCeilingDenial::Stopped(cause))
        })
}
