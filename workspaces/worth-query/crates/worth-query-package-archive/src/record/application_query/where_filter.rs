//! An optional trailing extension preserves every unfiltered query's bytes.
//! A filter decodes as an untrusted draft and is readmitted by Foundation.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::{
    decode_budget::RecordDecodeAttempt,
    foundational_aspect, foundational_value,
    sequence::{decode_sequence, write_sequence},
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryWhereFilterOperand as Operand, ApplicationQueryWhereFilterProgram as Program,
    ApplicationQueryWhereFilterSource as Source,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

const MARKER: &[u8; 4] = b"WQWF";
const VERSION: u16 = 1;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    filter: Option<&Program>,
) -> Result<(), Denial> {
    let Some(filter) = filter else {
        return Ok(());
    };
    output.raw_bytes(MARKER)?;
    output.u16(VERSION)?;
    write_program_body(output, filter)
}

pub(super) fn write_program_body(
    output: &mut dyn BinaryEncodingSink,
    filter: &Program,
) -> Result<(), Denial> {
    write_sequence(output, filter.operands(), write_operand)?;
    let draft = filter.draft();
    output.u32(u32::try_from(draft.len()).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?)?;
    output.raw_bytes(draft)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Option<Program>, Denial> {
    if !input.remaining_starts_with(MARKER) {
        return Ok(None);
    }
    input.take(MARKER.len())?;
    if input.u16()? != VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    decode_program_body(input, budget).map(Some)
}

pub(super) fn decode_program_body(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Program, Denial> {
    let operands = decode_sequence(input, budget, 6, |input, _| decode_operand(input))?;
    let length = usize::try_from(input.u32()?).map_err(|_| Denial::new(Kind::Truncated))?;
    let draft = input.take(length)?;
    Program::from_untrusted(operands, draft).map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

fn write_operand(output: &mut dyn BinaryEncodingSink, operand: &Operand) -> Result<(), Denial> {
    output.text(operand.name())?;
    match operand.source() {
        Source::Field {
            entity,
            aspect,
            field,
            scalar_family,
            presence,
        } => {
            output.u8(0)?;
            output.text(entity)?;
            output.text(aspect)?;
            output.text(field)?;
            foundational_aspect::write_scalar_type(output, *scalar_family)?;
            output.u8(match presence {
                ApplicationFieldPresence::Required => 0,
                ApplicationFieldPresence::Optional => 1,
            })
        }
        Source::Parameter {
            parameter,
            scalar_family,
        }
        | Source::OptionalParameter {
            parameter,
            scalar_family,
        } => {
            output.u8(
                if matches!(operand.source(), Source::OptionalParameter { .. }) {
                    2
                } else {
                    1
                },
            )?;
            output.text(parameter)?;
            foundational_aspect::write_scalar_type(output, *scalar_family)
        }
        Source::Literal {
            scalar_family,
            value,
        } => {
            output.u8(3)?;
            foundational_aspect::write_scalar_type(output, *scalar_family)?;
            foundational_value::write_aspect_value(output, value)
        }
    }
}

fn decode_operand(input: &mut BinaryInput<'_>) -> Result<Operand, Denial> {
    let name = input.text()?.to_owned();
    let source = match input.u8()? {
        0 => Source::Field {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            presence: match input.u8()? {
                0 => ApplicationFieldPresence::Required,
                1 => ApplicationFieldPresence::Optional,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
        },
        1 => Source::Parameter {
            parameter: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
        },
        2 => Source::OptionalParameter {
            parameter: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
        },
        3 => Source::Literal {
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value: foundational_value::decode_aspect_value(input)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(Operand::from_untrusted(name, source))
}

#[cfg(test)]
mod tests;
