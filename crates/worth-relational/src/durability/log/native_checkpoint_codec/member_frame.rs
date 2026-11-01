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

/// Admit and emit the same complete frame through the original request.
pub(crate) fn encode_for_request(
    kind: MemberKind,
    value: &impl serde::Serialize,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    (
        RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    ),
    crate::durability::data::RelationalRecoveryStop,
> {
    let mut prefix = [0_u8; MAGIC.len() + 3];
    prefix[..MAGIC.len()].copy_from_slice(MAGIC);
    prefix[MAGIC.len()..MAGIC.len() + 2].copy_from_slice(&1_u16.to_be_bytes());
    prefix[MAGIC.len() + 2] = kind as u8;
    let bytes = crate::durability::artifact_payload_emission::request::encode_named(
        &prefix, value, request,
    )?;
    Ok((locator_for_request(bytes.bytes(), request)?, bytes))
}

fn locator_for_request(
    bytes: &[u8],
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<RelationalDurableMemberLocator, crate::durability::data::RelationalRecoveryStop> {
    use crate::durability::data::RelationalRecoveryStop as Stop;
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        digest.update(chunk);
    }
    let length = u64::try_from(bytes.len()).map_err(|_| {
        Stop::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
    })?;
    Ok(RelationalDurableMemberLocator {
        sha256: digest.finalize().into(),
        length,
    })
}

pub(crate) fn locator(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
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
