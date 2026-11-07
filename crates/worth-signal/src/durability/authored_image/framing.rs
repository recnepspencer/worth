use super::codec::Write;
use super::{
    SignalAuthoredImage, SignalAuthoredImageDenial as Denial, SIGNAL_AUTHORED_IMAGE_VERSION,
};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};

const MAGIC: &[u8; 8] = b"WSIGAUTH";
const HEADER: usize = 56;
const MAXIMUM: usize = 64 << 20;

pub(crate) fn encode(
    execution: &ExecutionRequest<'_, '_>,
    emit: &mut dyn FnMut(&mut Write<'_>) -> Result<(), Denial>,
) -> Result<SignalAuthoredImage, Denial> {
    let mut size = 0usize;
    emit(&mut Write::new(&mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|n| *n <= MAXIMUM - HEADER)
            .ok_or(Denial::SizeLimit)?;
        Ok(())
    }))?;
    let version = SIGNAL_AUTHORED_IMAGE_VERSION.get().to_le_bytes();
    let profile = (crate::data::aspect::MAX_ASPECTS as u32).to_le_bytes();
    let length = (size as u64).to_le_bytes();
    let mut checksum = Sha256::new();
    checksum.update(version);
    checksum.update(profile);
    checksum.update(length);
    emit(&mut Write::new(&mut |bytes| {
        checksum.update(bytes);
        Ok(())
    }))?;
    let checksum: [u8; 32] = checksum.finalize().into();
    let mut out = ExecutionRequestArrayBuilder::allocate(*execution, HEADER + size)
        .map_err(Denial::Backing)?;
    for field in [MAGIC.as_slice(), &version, &profile, &length, &checksum] {
        out.extend_from_slice(field).map_err(Denial::Backing)?;
    }
    emit(&mut Write::new(&mut |bytes| {
        out.extend_from_slice(bytes).map_err(Denial::Backing)
    }))?;
    Ok(SignalAuthoredImage::from_encoded(
        out.seal().map_err(Denial::Backing)?.into_immutable_bytes(),
    ))
}

pub(super) fn payload(bytes: &[u8]) -> Result<&[u8], Denial> {
    if bytes.len() > MAXIMUM {
        return Err(Denial::SizeLimit);
    }
    if bytes.len() < HEADER || &bytes[..8] != MAGIC {
        return Err(Denial::Malformed);
    }
    let produced =
        BoundaryProtocolVersion::try_new(u32::from_le_bytes(bytes[8..12].try_into().unwrap()))
            .map_err(|_| Denial::Malformed)?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        SIGNAL_AUTHORED_IMAGE_VERSION,
        SIGNAL_AUTHORED_IMAGE_VERSION,
    )
    .admit(produced)
    .map_err(Denial::UnsupportedVersion)?;
    let recorded_aspects = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
    let supported_aspects = crate::data::aspect::MAX_ASPECTS as u32;
    if recorded_aspects != supported_aspects {
        return Err(Denial::StorageProfileMismatch {
            recorded_aspects,
            supported_aspects,
        });
    }
    let size = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    if size != (bytes.len() - HEADER) as u64 {
        return Err(Denial::Malformed);
    }
    let mut checksum = Sha256::new();
    checksum.update(&bytes[8..24]);
    checksum.update(&bytes[HEADER..]);
    let observed: [u8; 32] = checksum.finalize().into();
    if observed.as_slice() != &bytes[24..HEADER] {
        return Err(Denial::ChecksumMismatch);
    }
    Ok(&bytes[HEADER..])
}
