//! V7 carries exact Foundation drafts and their native source bindings.
mod signed_evidence;
use super::{BinaryEncodingSink, BinaryInput, Denial, Kind, RecordDecodeAttempt};
use crate::record::{
    decode_budget,
    sequence::{decode_sequence, write_sequence},
};
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
        write_sequence(output, requirement.operands(), |output, operand| {
            output.text(operand.name())?;
            match operand.source() {
                Source::SignedEvidence(source) => {
                    output.u16(4)?;
                    signed_evidence::write(output, source)
                }
                Source::PrincipalIdentity {
                    scope,
                    mutation_binding,
                    principal_binding,
                    field,
                    binding,
                    scalar_family,
                } => {
                    output.u16(3)?;
                    output.text(scope)?;
                    output.text(mutation_binding)?;
                    output.text(principal_binding)?;
                    output.text(field.entity())?;
                    output.text(field.aspect())?;
                    output.text(field.field())?;
                    output.text(binding.as_str())?;
                    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)
                }
                Source::Field {
                    field,
                    binding,
                    scalar_family,
                    presence,
                    presence_only,
                } => {
                    output.u16(1)?;
                    output.text(field.entity())?;
                    output.text(field.aspect())?;
                    output.text(field.field())?;
                    output.text(binding.as_str())?;
                    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                    output.u8(u8::from(*presence == ApplicationFieldPresence::Optional))?;
                    output.u8(u8::from(*presence_only))
                }
                Source::RelatedItemCount {
                    relation,
                    from,
                    to,
                    direction,
                } => {
                    output.u16(2)?;
                    output.text(relation)?;
                    output.text(from)?;
                    output.text(to)?;
                    output.u8(u8::from(*direction == Direction::Incoming))
                }
            }
        })?;
        output.u8(u8::from(requirement.applicability().is_some()))?;
        if let Some(expression) = requirement.applicability() {
            write_draft(output, expression.draft())?;
        }
        write_draft(output, requirement.satisfaction().draft())
    })
}

fn write_draft(output: &mut dyn BinaryEncodingSink, bytes: &[u8]) -> Result<(), Denial> {
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
        let operands = decode_sequence(input, budget, 12, |input, _| {
            let name = input.text()?.to_owned();
            let source = match input.u16()? {
                4 if version >= 16 => Source::SignedEvidence(signed_evidence::decode(input)?),
                3 if version >= 13 => Source::PrincipalIdentity {
                    scope: input.text()?.to_owned(),
                    mutation_binding: input.text()?.to_owned(),
                    principal_binding: input.text()?.to_owned(),
                    field: ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    ),
                    binding: WorthQueryPortableTypeIdentity::from_untrusted(
                        input.text()?.to_owned(),
                    ),
                    scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
                },
                1 => {
                    let field = ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    );
                    let binding =
                        WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
                    let scalar_family =
                        super::super::foundational_aspect::decode_scalar_type(input)?;
                    let presence = if boolean(input)? {
                        ApplicationFieldPresence::Optional
                    } else {
                        ApplicationFieldPresence::Required
                    };
                    let presence_only = boolean(input)?;
                    Source::Field {
                        field,
                        binding,
                        scalar_family,
                        presence,
                        presence_only,
                    }
                }
                2 => Source::RelatedItemCount {
                    relation: input.text()?.to_owned(),
                    from: input.text()?.to_owned(),
                    to: input.text()?.to_owned(),
                    direction: if boolean(input)? {
                        Direction::Incoming
                    } else {
                        Direction::Outgoing
                    },
                },
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            };
            Ok(Operand::from_untrusted(name, source))
        })?;
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

fn draft(
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
