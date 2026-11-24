//! Current-version required-input meaning follows genuine restoration identity.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::producing_implementation::RequiredOutputProducingImplementationOwnerPort as _;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::*;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

pub struct CopiedOutputInputs {
    pub meanings: Vec<RequiredOutputPublicationMeaning>,
    pub memory: Option<ExecutionMemoryReservation>,
}

pub fn copy(
    inputs: &[RequiredOutputPublicationMeaning],
    correspondence: &[(EntityId, EntityId)],
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CopiedOutputInputs, TransactionCommitError> {
    let bytes = inputs
        .iter()
        .fold(
            inputs
                .len()
                .checked_mul(std::mem::size_of::<RequiredOutputPublicationMeaning>()),
            |bytes, input| bytes?.checked_add(input.family.len()),
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            origin_request(crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    let memory = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(|cause| origin_request(crate::execution::denial::lease_denial(cause)))?;
    let mut meanings = Vec::with_capacity(inputs.len());
    for input in inputs {
        context
            .checkpoint((input.family.len() + correspondence.len() + 65) as u64)
            .map_err(origin_request)?;
        let mut input = input.clone();
        if let Some((_, restored)) = correspondence
            .iter()
            .find(|(old, _)| *old == input.source_scope)
        {
            input.source_scope = *restored;
        }
        meanings.push(input);
    }
    let comparisons = meanings
        .len()
        .saturating_mul(usize::BITS as usize - meanings.len().leading_zeros() as usize + 1);
    let width = meanings
        .iter()
        .map(|input| input.family.len())
        .max()
        .unwrap_or(0)
        .saturating_add(65);
    context
        .checkpoint(comparisons.saturating_mul(width) as u64)
        .map_err(origin_request)?;
    meanings.sort_by(|a, b| key(a).cmp(&key(b)));
    Ok(CopiedOutputInputs {
        meanings,
        memory: Some(memory),
    })
}

pub(super) fn validate(
    origin: &CanonicalRequiredOutputOrigin,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), crate::history::data::CodecRetirementClosureStop> {
    let mut previous = None;
    for input in &origin.inputs {
        crate::history::data::codec_retirement_closure::charge(
            context,
            input.family.len() + origin.meaning.family.len() + 65,
        )?;
        if !input.implementation.valid()
            || input.family.is_empty()
            || origin.meaning.same_route(input)
            || previous.is_some_and(|previous| key(previous) >= key(input))
        {
            return Err("invalid canonical required-input route".into());
        }
        previous = Some(input);
    }
    Ok(())
}

fn key(
    input: &RequiredOutputPublicationMeaning,
) -> (&str, EntityId, worth_foundational::CanonicalDigestId) {
    (&input.family, input.source_scope, input.source_partition)
}
