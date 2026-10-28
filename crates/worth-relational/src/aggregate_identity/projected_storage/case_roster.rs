//! Canonical nonempty cases grouped on one real carrier-to-aggregate edge.
use super::super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::domain_text::{decode_domain, encode_domain, take_hex, take_text};
use crate::projected_member_domain::ProjectedMemberDomainHandle;
use std::fmt::Write;
use worth_execution::MapKernelFailure;

const PREFIX: &str = "WORTH.PROJECTED.CASES.1:";

pub(in crate::aggregate_identity) fn encode_cases(
    domain: &ProjectedMemberDomainHandle,
    cases: &[&str],
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<String> {
    validate_cases(domain, cases.iter().copied(), cases.len(), work)?;
    let domain_text = encode_domain(&domain.parts().identity, work)?;
    let bytes = cases.iter().try_fold(
        PREFIX
            .len()
            .checked_add(16)
            .and_then(|n| n.checked_add(domain_text.len()))
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
        |bytes, case| {
            bytes
                .checked_add(16)
                .and_then(|n| n.checked_add(case.len()))
                .ok_or(MapKernelFailure::ResultCapacityExceeded)
        },
    )?;
    work.checkpoint(bytes as u64)?;
    work.retain(bytes as u64)?;
    let mut output = String::new();
    output
        .try_reserve_exact(bytes)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    output.push_str(PREFIX);
    output.push_str(&domain_text);
    write!(output, "{:016x}", cases.len()).expect("writing a reserved String");
    for case in cases {
        write!(output, "{:016x}{case}", case.len()).expect("writing a reserved String");
    }
    Ok(output)
}

pub(in crate::aggregate_identity) fn decode_cases<'a>(
    encoded: &'a str,
    domain: &ProjectedMemberDomainHandle,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<&'a str>> {
    work.checkpoint(encoded.len() as u64)?;
    let invalid = || MapKernelFailure::Domain(PlanDenial::InvalidStoredIdentity);
    let remaining = encoded.strip_prefix(PREFIX).ok_or_else(invalid)?;
    let mut remaining = decode_domain(remaining, &domain.parts().identity, work)?;
    let count = usize::try_from(take_hex(&mut remaining, 16).ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    // Every case needs a frame even before its content. Reject hostile counts
    // before allocating, and never accept more than the declared case roster.
    if count == 0 || count > domain.parts().cases.len() || count > remaining.len() / 16 {
        return Err(invalid());
    }
    let bytes = count
        .checked_mul(size_of::<&str>())
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    work.retain(bytes as u64)?;
    let mut cases = Vec::new();
    cases
        .try_reserve_exact(count)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    for _ in 0..count {
        cases.push(take_text(&mut remaining).ok_or_else(invalid)?);
    }
    if !remaining.is_empty() {
        return Err(invalid());
    }
    validate_cases(domain, cases.iter().copied(), count, work)?;
    Ok(cases)
}

fn validate_cases<'a>(
    domain: &ProjectedMemberDomainHandle,
    cases: impl Iterator<Item = &'a str>,
    count: usize,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    if count == 0 || count > domain.parts().cases.len() {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let levels = u64::from(usize::BITS - domain.parts().cases.len().max(1).leading_zeros());
    let mut previous = None;
    for case in cases {
        let units = (case.len() as u64)
            .checked_mul(levels)
            .and_then(|n| n.checked_add(case.len() as u64))
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        work.checkpoint(units)?;
        if previous.is_some_and(|previous| previous >= case) || domain.case(case).is_none() {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        }
        previous = Some(case);
    }
    Ok(())
}
