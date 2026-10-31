mod fields;
mod native_component;
use super::artifact::WORLD_ARTIFACT_VERSION;
use super::{WorldDurableFrameDenial as Denial, WorldDurableMediumDenial, WorldRecord};
use fields::*;
pub(super) use fields::{association as write_association, Read};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion, HomeIdentity, HomeIncarnation,
};
const MAGIC: &[u8; 8] = b"WWRLDART";
const HEADER: usize = 52;
const MAXIMUM: usize = 64 << 10;
pub(crate) fn encode(
    record: &WorldRecord,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    encode_payload(MAGIC, execution, &mut |out| write_record(record, out))
}
pub(super) fn encode_payload(
    magic: &[u8; 8],
    execution: &ExecutionRequest<'_, '_>,
    payload: &mut dyn FnMut(&mut Emit<'_>) -> Result<(), Denial>,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    let mut digest = Sha256::new();
    let mut size = 0usize;
    payload(&mut |bytes| {
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= MAXIMUM - HEADER)
            .ok_or(Denial::SizeLimit)?;
        digest.update(bytes);
        Ok(())
    })
    .map_err(WorldDurableMediumDenial::Frame)?;
    let digest: [u8; 32] = digest.finalize().into();
    let mut builder = ExecutionRequestArrayBuilder::allocate(*execution, HEADER + size)
        .map_err(WorldDurableMediumDenial::Backing)?;
    for bytes in [
        magic.as_slice(),
        WORLD_ARTIFACT_VERSION.get().to_le_bytes().as_slice(),
        (size as u64).to_le_bytes().as_slice(),
        digest.as_slice(),
    ] {
        for byte in bytes {
            builder
                .push(*byte)
                .map_err(WorldDurableMediumDenial::Backing)?;
        }
    }
    let mut stopped = None;
    payload(&mut |bytes| {
        for byte in bytes {
            if let Err(stop) = builder.push(*byte) {
                stopped = Some(stop);
                return Err(Denial::Malformed);
            }
        }
        Ok(())
    })
    .map_err(|denial| match stopped {
        Some(stop) => WorldDurableMediumDenial::Backing(stop),
        None => WorldDurableMediumDenial::Frame(denial),
    })?;
    builder.seal().map_err(WorldDurableMediumDenial::Backing)
}
pub(crate) fn decode(bytes: &[u8], home: HomeIdentity) -> Result<WorldRecord, Denial> {
    let payload = payload(bytes, MAGIC)?;
    let mut read = Read::new(payload, home);
    let tag = read.fixed::<1>()?[0];
    let record = match tag {
        1 => WorldRecord::Owner {
            owner: read.owner()?,
        },
        2 => {
            let head = read.head()?;
            let bootstrap =
                worth_foundational::facade::DurableWorldBootstrapAttemptIdentity::from_recorded(
                    head.branch.owner(),
                    read.number()?,
                );
            WorldRecord::Branch {
                head,
                bootstrap,
                program_revision: read.fixed()?,
                relational: read.recorded_component()?,
            }
        }
        3 => WorldRecord::OpenAttempt(read.open()?),
        4 => {
            let open = read.open()?;
            let relational = read.evidence()?;
            let publishing_incarnation =
                HomeIncarnation::from_recorded(read.number()?).ok_or(Denial::Malformed)?;
            WorldRecord::Occurrence {
                open,
                relational,
                publishing_incarnation,
            }
        }
        5 | 6 => {
            let association = read.association()?;
            let custody = worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(association.attempt().owner(), read.number()?);
            if tag == 5 {
                WorldRecord::Released {
                    association,
                    custody,
                }
            } else {
                WorldRecord::Continued {
                    association,
                    custody,
                }
            }
        }
        _ => return Err(Denial::Malformed),
    };
    read.finished()?;
    Ok(record)
}
pub(super) fn payload<'a>(bytes: &'a [u8], magic: &[u8; 8]) -> Result<&'a [u8], Denial> {
    if bytes.len() < HEADER || bytes.len() > MAXIMUM || &bytes[..8] != magic {
        return Err(Denial::Malformed);
    }
    let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(
        bytes[8..12].try_into().map_err(|_| Denial::Malformed)?,
    ))
    .map_err(|_| Denial::Malformed)?;
    BoundaryProtocolCompatibilityWindow::inclusive(WORLD_ARTIFACT_VERSION, WORLD_ARTIFACT_VERSION)
        .admit(version)
        .map_err(Denial::UnsupportedVersion)?;
    let size = u64::from_le_bytes(bytes[12..20].try_into().map_err(|_| Denial::Malformed)?);
    let payload = &bytes[HEADER..];
    if size != payload.len() as u64 {
        return Err(Denial::Malformed);
    }
    let digest: [u8; 32] = Sha256::digest(payload).into();
    if digest.as_slice() != &bytes[20..HEADER] {
        return Err(Denial::DigestMismatch);
    }
    Ok(payload)
}
fn write_record(record: &WorldRecord, out: &mut Emit<'_>) -> Result<(), Denial> {
    match record {
        WorldRecord::Owner { owner: value } => {
            out(&[1])?;
            owner(out, *value)
        }
        WorldRecord::Branch {
            head: value,
            bootstrap,
            program_revision,
            relational,
        } => {
            out(&[2])?;
            head(out, value)?;
            number(out, bootstrap.ordinal())?;
            out(program_revision)?;
            native_component::emit(out, relational)
        }
        WorldRecord::OpenAttempt(value) => {
            out(&[3])?;
            open(out, value)
        }
        WorldRecord::Occurrence {
            open: value,
            relational,
            publishing_incarnation,
        } => {
            out(&[4])?;
            open(out, value)?;
            evidence(out, relational)?;
            number(out, publishing_incarnation.get())
        }
        WorldRecord::Released {
            association: value,
            custody,
        } => {
            out(&[5])?;
            association(out, value)?;
            number(out, custody.ordinal())
        }
        WorldRecord::Continued {
            association: value,
            custody,
        } => {
            out(&[6])?;
            association(out, value)?;
            number(out, custody.ordinal())
        }
    }
}
