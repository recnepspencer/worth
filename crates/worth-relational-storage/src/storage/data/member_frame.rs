//! Non-region members share a versioned, kind-tagged frame. The directory's
//! SHA-256 covers the entire frame, including its interpretation.
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use sha2::{Digest, Sha256};

pub const MAGIC: &[u8] = b"WORTH-REL-MBR1\n";

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MemberKind {
    Symbols = 0,
    Lineage = 1,
    DerivedIndex = 2,
    LineageEvents = 3,
}

pub fn encode(
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

pub fn locator(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}

/// The same physical identity, with original controls during each actual hash chunk.
pub fn locator_for_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
) -> Result<
    RelationalDurableMemberLocator,
    crate::execution::denial_cause::RelationalExecutionDenialCause,
> {
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
pub fn locator_in_stage(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    bytes: &[u8],
) -> Result<
    RelationalDurableMemberLocator,
    crate::execution::denial_cause::RelationalExecutionDenialCause,
> {
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(4096) {
        crate::execution::request_stage::checkpoint(context, chunk.len() as u64)
            .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
        digest.update(chunk);
    }
    crate::execution::request_stage::checkpoint(context, 0)
        .map_err(crate::execution::denial_cause::RelationalExecutionDenialCause::from)?;
    Ok(RelationalDurableMemberLocator {
        sha256: digest.finalize().into(),
        length: bytes.len() as u64,
    })
}

fn decode_verified_header(bytes: &[u8], kind: MemberKind) -> Result<&[u8], Cause> {
    let frame = bytes.strip_prefix(MAGIC).ok_or(Cause::Damaged)?;
    let (header, payload) = frame.split_at_checked(3).ok_or(Cause::Damaged)?;
    if header[..2] != 1_u16.to_be_bytes() || header[2] != kind as u8 {
        return Err(Cause::Damaged);
    }
    Ok(payload)
}

pub fn decode_for_request<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &'a [u8],
    expected: RelationalDurableMemberLocator,
    kind: MemberKind,
    subject: crate::storage::data::storage_unavailable::RelationalStorageSubject,
) -> Result<&'a [u8], crate::durability::data::recovery_stop::RelationalRecoveryStop> {
    let actual = locator_for_request(execution, bytes)
        .map_err(crate::durability::data::recovery_stop::RelationalRecoveryStop::Request)?;
    if actual != expected {
        return Err(crate::storage::data::storage_unavailable::RelationalStorageUnavailable {
            subject,
            cause: Cause::IntegrityMismatch,
        }
        .into());
    }
    decode_verified_header(bytes, kind).map_err(|cause| {
        crate::storage::data::storage_unavailable::RelationalStorageUnavailable { subject, cause }.into()
    })
}

pub fn decode(
    bytes: &[u8],
    expected: RelationalDurableMemberLocator,
    kind: MemberKind,
) -> Result<&[u8], Cause> {
    if locator(bytes) != expected {
        return Err(Cause::IntegrityMismatch);
    }
    decode_verified_header(bytes, kind)
}
