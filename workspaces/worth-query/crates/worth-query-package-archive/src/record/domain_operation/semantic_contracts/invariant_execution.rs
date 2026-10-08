use super::{read_usize, unsupported, write_usize};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};
use std::num::NonZeroU32;
use worth_query_installation::facade::*;

fn dense(requirement: &WorthQueryInstalledInvariantExecutionRequirement) -> bool {
    matches!((requirement.max_state_facts(), requirement.max_work_units()),
        (Some(state), Some(work)) if state > 0 && work > 0)
}

pub(in crate::record::domain_operation) fn write_invariant_execution(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryInvariantExecutionContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryInvariantExecutionContract::NotRequired => output.u16(1),
        WorthQueryInvariantExecutionContract::Declared { requirements } => {
            let legacy = requirements.iter().all(dense);
            output.u16(if legacy { 2 } else { 3 })?;
            write_sequence(output, requirements, |output, requirement| {
                output.text(requirement.slot())?;
                output.text(requirement.family())?;
                output.u32(requirement.version().get())?;
                output.u16(match requirement.enforcement() {
                    WorthQueryInvariantEnforcement::Blocking => 1,
                    WorthQueryInvariantEnforcement::Advisory => 2,
                })?;
                output.text(requirement.executor_role())?;
                write_sequence(
                    output,
                    requirement.state_load_families(),
                    |output, family| output.text(family),
                )?;
                if legacy {
                    match (requirement.max_state_facts(), requirement.max_work_units()) {
                        (Some(state), Some(work)) => {
                            write_usize(output, state)?;
                            output.u64(work)
                        }
                        _ => Err(Denial::new(Kind::InvalidRecordShape)),
                    }
                } else {
                    write_optional_state(output, requirement.max_state_facts())?;
                    write_optional_work(output, requirement.max_work_units())
                }
            })
        }
    }
}

pub(in crate::record::domain_operation) fn decode_invariant_execution(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryInvariantExecutionContract, Denial> {
    let tag = input.u16()?;
    if tag == 1 {
        return Ok(WorthQueryInvariantExecutionContract::NotRequired);
    }
    if tag != 2 && tag != 3 {
        return unsupported();
    }
    let requirements = decode_sequence(
        input,
        budget,
        if tag == 2 { 36 } else { 26 },
        |input, budget| {
            let slot = input.text()?.to_owned();
            let family = input.text()?.to_owned();
            let version = NonZeroU32::new(input.u32()?)
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
            let enforcement = match input.u16()? {
                1 => WorthQueryInvariantEnforcement::Blocking,
                2 => WorthQueryInvariantEnforcement::Advisory,
                _ => return unsupported(),
            };
            let executor = input.text()?.to_owned();
            let loads = decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
            require_canonical_sequence(&loads)?;
            if tag == 2 {
                WorthQueryInstalledInvariantExecutionRequirement::new(
                    slot,
                    family,
                    version,
                    enforcement,
                    executor,
                    loads,
                    read_usize(input)?,
                    input.u64()?,
                )
            } else {
                WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
                    slot,
                    family,
                    version,
                    enforcement,
                    executor,
                    loads,
                    read_optional_state(input)?,
                    read_optional_work(input)?,
                )
            }
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))
        },
    )?;
    if tag == 3 && requirements.iter().all(dense) {
        return Err(Denial::new(Kind::NonCanonicalRecordSequence));
    }
    if requirements
        .windows(2)
        .any(|pair| pair[0].slot() >= pair[1].slot())
    {
        return Err(Denial::new(Kind::NonCanonicalRecordSequence));
    }
    WorthQueryInvariantExecutionContract::declared(requirements)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

fn write_optional_state(
    output: &mut dyn BinaryEncodingSink,
    value: Option<usize>,
) -> Result<(), Denial> {
    match value {
        None => output.u16(1),
        Some(value) => {
            output.u16(2)?;
            write_usize(output, value)
        }
    }
}
fn write_optional_work(
    output: &mut dyn BinaryEncodingSink,
    value: Option<u64>,
) -> Result<(), Denial> {
    match value {
        None => output.u16(1),
        Some(value) => {
            output.u16(2)?;
            output.u64(value)
        }
    }
}
fn read_optional_state(input: &mut BinaryInput<'_>) -> Result<Option<usize>, Denial> {
    match input.u16()? {
        1 => Ok(None),
        2 => Ok(Some(read_usize(input)?)),
        _ => unsupported(),
    }
}
fn read_optional_work(input: &mut BinaryInput<'_>) -> Result<Option<u64>, Denial> {
    match input.u16()? {
        1 => Ok(None),
        2 => Ok(Some(input.u64()?)),
        _ => unsupported(),
    }
}
