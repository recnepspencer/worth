//! One strict length framing for persisted projected-domain coordinates.
use super::super::funding::{AggregateWork, BuildResult, PlanDenial};
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use std::fmt::Write;
use worth_execution::MapKernelFailure;

const PREFIX: &str = "WORTH.PROJECTED.DOMAIN.1:";

pub(in crate::aggregate_identity) fn encode_domain(
    identity: &ProjectedMemberDomainIdentity,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<String> {
    let bytes = PREFIX
        .len()
        .checked_add(40)
        .and_then(|n| n.checked_add(identity.schema.0.len()))
        .and_then(|n| n.checked_add(identity.name.len()))
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    work.checkpoint(bytes as u64)?;
    work.retain(bytes as u64)?;
    let mut output = String::new();
    output
        .try_reserve_exact(bytes)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    output.push_str(PREFIX);
    write!(
        output,
        "{:016x}{}{:08x}{:016x}{}",
        identity.schema.0.len(),
        identity.schema.0,
        identity.version.0,
        identity.name.len(),
        identity.name
    )
    .expect("writing a reserved String");
    Ok(output)
}

pub(in crate::aggregate_identity) fn decode_domain<'a>(
    encoded: &'a str,
    identity: &ProjectedMemberDomainIdentity,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<&'a str> {
    work.checkpoint(encoded.len() as u64)?;
    let invalid = || MapKernelFailure::Domain(PlanDenial::InvalidStoredIdentity);
    let mut remaining = encoded.strip_prefix(PREFIX).ok_or_else(invalid)?;
    let schema = take_text(&mut remaining).ok_or_else(invalid)?;
    let version = take_hex(&mut remaining, 8).ok_or_else(invalid)?;
    let name = take_text(&mut remaining).ok_or_else(invalid)?;
    if schema != identity.schema.0
        || version != u64::from(identity.version.0)
        || name != identity.name
    {
        return Err(invalid());
    }
    Ok(remaining)
}

pub(super) fn take_text<'a>(input: &mut &'a str) -> Option<&'a str> {
    let bytes = usize::try_from(take_hex(input, 16)?).ok()?;
    let text = input.get(..bytes)?;
    *input = input.get(bytes..)?;
    Some(text)
}

pub(super) fn take_hex(input: &mut &str, width: usize) -> Option<u64> {
    let text = input.get(..width)?;
    let mut number = 0_u64;
    for byte in text.bytes() {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => return None,
        };
        number = number.checked_mul(16)?.checked_add(u64::from(digit))?;
    }
    *input = input.get(width..)?;
    Some(number)
}
