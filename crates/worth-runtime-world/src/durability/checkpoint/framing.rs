//! World1 is one bounded, owner-framed inline Home section.
use crate::durability::codec::fields::Emit;
use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};
#[cfg(test)]
mod tests;

const MAGIC: &[u8; 8] = b"WWRLDCP1";
const HEADER: usize = 52;
pub(super) const MAXIMUM: usize = 1 << 30;

pub(super) fn charge(request: &ExecutionRequest<'_, '_>, bytes: usize) -> Result<(), Denial> {
    let work = u64::try_from(bytes).map_err(|_| Frame::SizeLimit)?;
    request
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(Denial::Request)?
        .0
        .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))
}

pub(super) fn encode(
    request: &ExecutionRequest<'_, '_>,
    payload: &mut dyn FnMut(&mut Emit<'_>) -> Result<(), Denial>,
) -> Result<ExecutionImmutableBytes, Denial> {
    let mut digest = Sha256::new();
    let mut size = 0usize;
    let mut stop = None;
    payload(&mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= MAXIMUM - HEADER)
            .ok_or(Frame::SizeLimit)?;
        if let Err(cause) = charge(request, bytes.len()) {
            stop = Some(cause);
            return Err(Frame::Malformed);
        }
        digest.update(bytes);
        Ok(())
    })
    .map_err(|cause| stop.unwrap_or(cause))?;
    let digest: [u8; 32] = digest.finalize().into();
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(*request, HEADER + size).map_err(Denial::Backing)?;
    for bytes in [
        MAGIC.as_slice(),
        super::WORLD_CHECKPOINT_VERSION
            .get()
            .to_le_bytes()
            .as_slice(),
        (size as u64).to_le_bytes().as_slice(),
        digest.as_slice(),
    ] {
        builder.extend_from_slice(bytes).map_err(Denial::Backing)?;
    }
    let mut stop = None;
    payload(&mut |bytes| {
        if let Err(cause) = charge(request, bytes.len()) {
            stop = Some(cause);
            return Err(Frame::Malformed);
        }
        builder.extend_from_slice(bytes).map_err(|cause| {
            stop = Some(Denial::Backing(cause));
            Frame::Malformed
        })
    })
    .map_err(|cause| stop.unwrap_or(cause))?;
    builder
        .seal()
        .map(|array| array.into_immutable_bytes())
        .map_err(Denial::Backing)
}

pub(super) fn payload<'a>(
    bytes: &'a [u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<&'a [u8], Denial> {
    inspect(bytes, &mut |bytes| charge(request, bytes))
}
fn inspect<'a>(
    bytes: &'a [u8],
    charge: &mut dyn FnMut(usize) -> Result<(), Denial>,
) -> Result<&'a [u8], Denial> {
    if bytes.len() < HEADER || bytes.len() > MAXIMUM || &bytes[..8] != MAGIC {
        return Err(Frame::Malformed.into());
    }
    let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
        bytes[8..12].try_into().map_err(|_| Frame::Malformed)?,
    ))
    .map_err(|_| Frame::Malformed)?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        super::WORLD_CHECKPOINT_VERSION,
        super::WORLD_CHECKPOINT_VERSION,
    )
    .admit(version)
    .map_err(|cause| Denial::Frame(Frame::UnsupportedVersion(cause)))?;
    let size = u64::from_le_bytes(bytes[12..20].try_into().map_err(|_| Frame::Malformed)?);
    let payload = &bytes[HEADER..];
    if size != payload.len() as u64 {
        return Err(Frame::Malformed.into());
    }
    let mut digest = Sha256::new();
    for chunk in payload.chunks(64 << 10) {
        charge(chunk.len())?;
        digest.update(chunk);
    }
    if digest.finalize().as_slice() != &bytes[20..HEADER] {
        return Err(Frame::DigestMismatch.into());
    }
    Ok(payload)
}
