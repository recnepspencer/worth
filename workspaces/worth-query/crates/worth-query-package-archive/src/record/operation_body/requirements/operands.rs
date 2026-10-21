use super::*;
pub(in crate::record::operation_body) fn write_operands(
    output: &mut dyn BinaryEncodingSink,
    operands: &[Operand],
) -> Result<(), Denial> {
    write_sequence(output, operands, |output, operand| {
        output.text(operand.name())?;
        match operand.source() {
            Source::QueryContext {
                mutation_binding,
                query,
            } => {
                output.u16(7)?;
                output.text(mutation_binding)?;
                crate::record::application_query::write(output, query)
            }
            Source::RelatedFieldValues {
                relation,
                from,
                to,
                field,
                binding,
                scalar_family,
                presence,
            } => {
                output.u16(6)?;
                for text in [
                    relation.as_str(),
                    from.as_str(),
                    to.as_str(),
                    field.entity(),
                    field.aspect(),
                    field.field(),
                    binding.as_str(),
                ] {
                    output.text(text)?;
                }
                super::super::super::foundational_aspect::write_scalar_type(
                    output,
                    *scalar_family,
                )?;
                output.u8(u8::from(*presence == ApplicationFieldPresence::Optional))
            }
            Source::SignedEvidence(source) => {
                output.u16(4)?;
                signed_evidence::write(output, source)
            }
            Source::RelatedSignedEvidence {
                relation,
                from,
                to,
                evidence,
            } => {
                output.u16(5)?;
                output.text(relation)?;
                output.text(from)?;
                output.text(to)?;
                signed_evidence::write(output, evidence)
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
                super::super::super::foundational_aspect::write_scalar_type(output, *scalar_family)
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
                super::super::super::foundational_aspect::write_scalar_type(
                    output,
                    *scalar_family,
                )?;
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
    })
}
pub(in crate::record::operation_body) fn decode_operands(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<Operand>, Denial> {
    decode_sequence(input, budget, 12, |input, budget| {
        let name = input.text()?.to_owned();
        let source = match input.u16()? {
            7 if version >= 25 => Source::QueryContext {
                mutation_binding: input.text()?.to_owned(),
                query: Box::new(crate::record::application_query::decode_nested(
                    input, budget,
                )?),
            },
            6 if version >= 24 => Source::RelatedFieldValues {
                relation: input.text()?.to_owned(),
                from: input.text()?.to_owned(),
                to: input.text()?.to_owned(),
                field: ApplicationFieldBindingLocus::new(
                    input.text()?,
                    input.text()?,
                    input.text()?,
                ),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned()),
                scalar_family: super::super::super::foundational_aspect::decode_scalar_type(input)?,
                presence: if boolean(input)? {
                    ApplicationFieldPresence::Optional
                } else {
                    ApplicationFieldPresence::Required
                },
            },
            5 if version >= 18 => Source::RelatedSignedEvidence {
                relation: input.text()?.to_owned(),
                from: input.text()?.to_owned(),
                to: input.text()?.to_owned(),
                evidence: signed_evidence::decode(input)?,
            },
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
                binding: WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned()),
                scalar_family: super::super::super::foundational_aspect::decode_scalar_type(input)?,
            },
            1 => {
                let field =
                    ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
                let binding =
                    WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
                let scalar_family =
                    super::super::super::foundational_aspect::decode_scalar_type(input)?;
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
    })
}
