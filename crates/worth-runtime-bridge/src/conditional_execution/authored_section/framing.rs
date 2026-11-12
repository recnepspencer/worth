//! Version and checksum admission, independently owned by Bridge.
use super::{
    fields::{Reader, Writer},
    BridgeAuthoredSectionDenial as D, BRIDGE_AUTHORED_SECTION_VERSION,
};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};

const MAGIC: &[u8; 8] = b"WBRGAUTH";
const HEADER: usize = 52;
pub(super) const MAXIMUM_BYTES: usize = 16 << 20;
pub(super) fn raw(
    execution: &ExecutionRequest<'_, '_>,
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), D>,
) -> Result<ExecutionImmutableBytes, D> {
    let charge = |bytes| work(execution, bytes);
    let mut size = 0usize;
    emit(&mut Writer::new(
        &mut |bytes| {
            size = size
                .checked_add(bytes.len())
                .filter(|size| *size <= MAXIMUM_BYTES)
                .ok_or(D::SizeLimit)?;
            Ok(())
        },
        &charge,
    ))?;
    let mut out = ExecutionRequestArrayBuilder::allocate(*execution, size).map_err(D::Backing)?;
    emit(&mut Writer::new(
        &mut |bytes| out.extend_from_slice(bytes).map_err(D::Backing),
        &charge,
    ))?;
    Ok(out.seal().map_err(D::Backing)?.into_immutable_bytes())
}

pub(super) fn encode(
    execution: &ExecutionRequest<'_, '_>,
    emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), D>,
) -> Result<ExecutionImmutableBytes, D> {
    let charge = |bytes| work(execution, bytes);
    let mut size = 0usize;
    emit(&mut Writer::new(
        &mut |bytes| {
            size = size
                .checked_add(bytes.len())
                .filter(|size| *size <= MAXIMUM_BYTES - HEADER)
                .ok_or(D::SizeLimit)?;
            Ok(())
        },
        &charge,
    ))?;
    let version = BRIDGE_AUTHORED_SECTION_VERSION.get().to_le_bytes();
    let length = (size as u64).to_le_bytes();
    work(execution, HEADER + version.len() + length.len())?;
    let mut hash = Sha256::new();
    hash.update(version);
    hash.update(length);
    emit(&mut Writer::new(
        &mut |bytes| {
            hash.update(bytes);
            Ok(())
        },
        &charge,
    ))?;
    let hash: [u8; 32] = hash.finalize().into();
    let mut out =
        ExecutionRequestArrayBuilder::allocate(*execution, HEADER + size).map_err(D::Backing)?;
    for field in [MAGIC.as_slice(), &version, &length, &hash] {
        out.extend_from_slice(field).map_err(D::Backing)?;
    }
    emit(&mut Writer::new(
        &mut |bytes| out.extend_from_slice(bytes).map_err(D::Backing),
        &charge,
    ))?;
    Ok(out.seal().map_err(D::Backing)?.into_immutable_bytes())
}
pub(super) fn payload(bytes: &[u8]) -> Result<Reader<'_>, D> {
    if bytes.len() > MAXIMUM_BYTES {
        return Err(D::SizeLimit);
    }
    if bytes.len() < HEADER || &bytes[..8] != MAGIC {
        return Err(D::Malformed);
    }
    let version =
        BoundaryProtocolVersion::try_new(u32::from_le_bytes(bytes[8..12].try_into().unwrap()))
            .map_err(|_| D::Malformed)?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        BRIDGE_AUTHORED_SECTION_VERSION,
        BRIDGE_AUTHORED_SECTION_VERSION,
    )
    .admit(version)
    .map_err(D::UnsupportedVersion)?;
    if u64::from_le_bytes(bytes[12..20].try_into().unwrap()) != (bytes.len() - HEADER) as u64 {
        return Err(D::Malformed);
    }
    let mut hash = Sha256::new();
    hash.update(&bytes[8..20]);
    hash.update(&bytes[HEADER..]);
    let hash: [u8; 32] = hash.finalize().into();
    if hash.as_slice() != &bytes[20..HEADER] {
        return Err(D::ChecksumMismatch);
    }
    Ok(Reader::new(&bytes[HEADER..]))
}
pub(super) fn copy(
    execution: &ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<ExecutionImmutableBytes, D> {
    work(execution, bytes.len().saturating_mul(2))?;
    if bytes.len() > MAXIMUM_BYTES {
        return Err(D::SizeLimit);
    }
    let mut out =
        ExecutionRequestArrayBuilder::allocate(*execution, bytes.len()).map_err(D::Backing)?;
    out.extend_from_slice(bytes).map_err(D::Backing)?;
    Ok(out.seal().map_err(D::Backing)?.into_immutable_bytes())
}
pub(super) fn work(execution: &ExecutionRequest<'_, '_>, amount: usize) -> Result<(), D> {
    if amount > MAXIMUM_BYTES.saturating_mul(3) {
        return Err(D::SizeLimit);
    }
    execution
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(amount as u64)
            } else {
                context.poll()
            }
        })
        .map_err(D::Request)?
        .0
        .map_err(|cause| D::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
