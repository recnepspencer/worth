//! V7 carries exact Foundation drafts and their native source bindings.
mod operands;
mod signed_evidence;
use super::{BinaryEncodingSink, BinaryInput, Denial, Kind, RecordDecodeAttempt};
use crate::record::{
    decode_budget,
    sequence::{decode_sequence, write_sequence},
};
pub(super) use operands::{decode_operands, write_operands};
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldPresence,
        ApplicationOperationBodyRequirementDirection as Direction,
        ApplicationOperationBodyRequirementOperand as Operand,
        ApplicationOperationBodyRequirementProgram as Requirement,
        ApplicationOperationBodyRequirementSource as Source,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    requirements: &[Requirement],
) -> Result<(), Denial> {
    write_sequence(output, requirements, |output, requirement| {
        output.text(requirement.identity())?;
        output.text(requirement.target_name())?;
        write_operands(output, requirement.operands())?;
        output.u8(u8::from(requirement.applicability().is_some()))?;
        if let Some(expression) = requirement.applicability() {
            write_draft(output, expression.draft())?;
        }
        write_draft(output, requirement.satisfaction().draft())
    })
}

pub(super) fn write_draft(output: &mut dyn BinaryEncodingSink, bytes: &[u8]) -> Result<(), Denial> {
    let length = u32::try_from(bytes.len()).map_err(|_| Denial::new(Kind::InvalidRecordLength))?;
    output.claim_nested_entries(length)?;
    output.u32(length)?;
    output.raw_bytes(bytes)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<Requirement>, Denial> {
    decode_sequence(input, budget, 16, |input, budget| {
        let identity = input.text()?.to_owned();
        let target = input.text()?.to_owned();
        let operands = decode_operands(input, budget, version)?;
        let applicability = if boolean(input)? {
            Some(draft(input, budget)?)
        } else {
            None
        };
        let satisfaction = draft(input, budget)?;
        Requirement::from_untrusted(identity, operands, applicability, satisfaction, target)
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))
    })
}

pub(super) fn draft(
    input: &mut BinaryInput<'_>,
    budget: &mut decode_budget::RecordDecodeAttempt,
) -> Result<Vec<u8>, Denial> {
    let length = input.u32()?;
    budget.claim_nested_entries(u64::from(length))?;
    Ok(input
        .take(usize::try_from(length).map_err(|_| Denial::new(Kind::InvalidRecordLength))?)?
        .to_vec())
}

fn boolean(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
