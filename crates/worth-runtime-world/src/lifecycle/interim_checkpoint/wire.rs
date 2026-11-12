//! Domain-separated movement rows and receiver binding in one current frame.
use super::*;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use worth_execution::ExecutionRequestArrayBuilder;
use worth_foundational::facade::RecordedCompositePublicationAttemptIdentity;
use worth_relational::facade::durability::CapturedNativeInterimAdmission;

type Emit<'a> = dyn FnMut(&[u8]) -> Result<(), Stop> + 'a;
const MAGIC: &[u8; 8] = b"WRWMREG1";
const VERSION: u32 = 1;
const MAXIMUM: usize = 1 << 30;

pub(super) fn digest(
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalDigestId, Stop> {
    let mut hash = Sha256::new();
    for chunk in bytes.chunks(64 << 10) {
        charge(request, chunk.len())?;
        hash.update(chunk);
    }
    Ok(CanonicalDigestId::new(hash.finalize().into()))
}
pub(super) fn row_digest(
    row: &CapturedRow,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalDigestId, Stop> {
    let mut hash = Sha256::new();
    hash.update(b"worth.world.interim.movement/1");
    write_row(
        row,
        &mut |bytes| {
            charge(request, bytes.len())?;
            hash.update(bytes);
            Ok(())
        },
        request,
    )?;
    Ok(CanonicalDigestId::new(hash.finalize().into()))
}
pub(super) fn encode(
    native: &CapturedNativeInterimAdmission,
    application: CanonicalDigestId,
    anchor: &MemoryProductBranchReference,
    anchor_component: &CapturedNativeInterimComponent,
    rows: &[CapturedRow],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(ExecutionImmutableBytes, CanonicalDigestId), Stop> {
    let payload = |out: &mut Emit<'_>| {
        out(b"worth.world.interim.receiver/1")?;
        out(native.coordinate().bytes())?;
        out(native.native_digest().bytes())?;
        out(&native.covered_position().to_le_bytes())?;
        out(&native.covered_transitions().to_le_bytes())?;
        out(application.bytes())?;
        reference(anchor, out)?;
        component(anchor_component, out, request)?;
        out(&(rows.len() as u64).to_le_bytes())?;
        for row in rows {
            out(row.digest.bytes())?;
            write_row(row, out, request)?;
        }
        Ok(())
    };
    let mut size = 0usize;
    let mut hash = Sha256::new();
    payload(&mut |bytes| {
        charge(request, bytes.len())?;
        size = size
            .checked_add(bytes.len())
            .filter(|size| *size <= MAXIMUM - 52)
            .ok_or(Stop::SizeOverflow)?;
        hash.update(bytes);
        Ok(())
    })?;
    let digest = CanonicalDigestId::new(hash.finalize().into());
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(*request, 52 + size).map_err(Stop::Backing)?;
    for bytes in [
        MAGIC.as_slice(),
        VERSION.to_le_bytes().as_slice(),
        (size as u64).to_le_bytes().as_slice(),
        digest.bytes().as_slice(),
    ] {
        builder.extend_from_slice(bytes).map_err(Stop::Backing)?;
    }
    payload(&mut |bytes| {
        charge(request, bytes.len())?;
        builder.extend_from_slice(bytes).map_err(Stop::Backing)
    })?;
    Ok((
        builder
            .seal()
            .map_err(Stop::Backing)?
            .into_immutable_bytes(),
        digest,
    ))
}
fn write_row(
    row: &CapturedRow,
    out: &mut Emit<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Stop> {
    reference(&row.meaning.before, out)?;
    reference(&row.meaning.after, out)?;
    match row.meaning.attempt {
        RecordedCompositePublicationAttemptIdentity::Memory { owner, ordinal } => {
            out(&owner.to_le_bytes())?;
            out(&ordinal.to_le_bytes())?;
        }
        RecordedCompositePublicationAttemptIdentity::Home(_) => return Err(Stop::BindingMismatch),
    }
    out(&[u8::from(row.bound_to_anchor)])?;
    component(&row.before, out, request)?;
    component(&row.after, out, request)
}
fn reference(value: &MemoryProductBranchReference, out: &mut Emit<'_>) -> Result<(), Stop> {
    out(&value.owner().to_le_bytes())?;
    out(&(value.branch().len() as u64).to_le_bytes())?;
    out(value.branch().as_bytes())?;
    for value in [
        value.incarnation(),
        value.generation(),
        value.selected_commit(),
    ] {
        out(&value.to_le_bytes())?;
    }
    Ok(())
}
fn component(
    value: &CapturedNativeInterimComponent,
    out: &mut Emit<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Stop> {
    let mut size = 0usize;
    json(value.description(), &mut |bytes| {
        charge(request, bytes.len())?;
        size = size.checked_add(bytes.len()).ok_or(Stop::SizeOverflow)?;
        Ok(())
    })?;
    out(&(size as u64).to_le_bytes())?;
    json(value.description(), out)
}
fn json(value: &impl serde::Serialize, out: &mut Emit<'_>) -> Result<(), Stop> {
    let mut stop = None;
    serde_json::to_writer(
        Emitter {
            out,
            stop: &mut stop,
        },
        value,
    )
    .map_err(|_| stop.unwrap_or(Stop::Malformed))
}
struct Emitter<'a, 'b> {
    out: &'a mut Emit<'b>,
    stop: &'a mut Option<Stop>,
}
impl Write for Emitter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match (self.out)(bytes) {
            Ok(()) => Ok(bytes.len()),
            Err(cause) => {
                *self.stop = Some(cause);
                Err(io::Error::other("World interim emission refused"))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
