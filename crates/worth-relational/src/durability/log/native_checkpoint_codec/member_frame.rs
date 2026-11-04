//! Non-region members share a versioned, kind-tagged frame. The directory's
//! SHA-256 covers the entire frame, including its interpretation.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use sha2::{Digest, Sha256};

const MAGIC: &[u8] = b"WORTH-REL-MBR1\n";

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum MemberKind {
    Symbols = 0,
    Lineage = 1,
    DerivedIndex = 2,
    LineageEvents = 3,
}

pub(crate) fn encode(
    kind: MemberKind,
    value: &impl serde::Serialize,
) -> Result<(RelationalDurableMemberLocator, Vec<u8>), Cause> {
    let payload = rmp_serde::to_vec_named(value).map_err(|_| Cause::Damaged)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(MAGIC.len() + 3 + payload.len())
        .map_err(|_| Cause::MemoryExhausted)?;
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&1_u16.to_be_bytes());
    bytes.push(kind as u8);
    bytes.extend_from_slice(&payload);
    Ok((locator(&bytes), bytes))
}

pub(crate) fn locator(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}

/// The same physical identity, with original controls during each actual hash chunk.
pub(crate) fn locator_for_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<RelationalDurableMemberLocator, crate::execution::RelationalExecutionDenialCause> {
    let mut answer = None;
    let handoff = crate::execution::request_stage::optional_stage(
        execution,
        |context, _| answer = Some(locator_in_stage(context, bytes)),
        |_| 0,
    );
    match handoff {
        Ok(()) => answer.expect("completed physical hash stage ran its callback"),
        Err(cause) => match answer {
            Some(Err(cause)) => Err(cause),
            _ => Err(cause),
        },
    }
}

/// Hash inside the caller's stage, one unit per hashed byte; opens no scope.
pub(crate) fn locator_in_stage(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    bytes: &[u8],
) -> Result<RelationalDurableMemberLocator, crate::execution::RelationalExecutionDenialCause> {
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(4096) {
        crate::execution::request_stage::checkpoint(context, chunk.len() as u64)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        digest.update(chunk);
    }
    crate::execution::request_stage::checkpoint(context, 0)
        .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
    Ok(RelationalDurableMemberLocator {
        sha256: digest.finalize().into(),
        length: bytes.len() as u64,
    })
}

pub(crate) fn decode(
    bytes: &[u8],
    expected: RelationalDurableMemberLocator,
    kind: MemberKind,
) -> Result<&[u8], Cause> {
    if locator(bytes) != expected {
        return Err(Cause::IntegrityMismatch);
    }
    let frame = bytes.strip_prefix(MAGIC).ok_or(Cause::Damaged)?;
    let (header, payload) = frame.split_at_checked(3).ok_or(Cause::Damaged)?;
    if header[..2] != 1_u16.to_be_bytes() || header[2] != kind as u8 {
        return Err(Cause::Damaged);
    }
    Ok(payload)
}
