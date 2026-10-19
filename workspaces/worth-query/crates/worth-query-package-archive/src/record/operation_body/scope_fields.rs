use super::*;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    steps: &[ApplicationOperationBodyScopeField],
) -> Result<(), Denial> {
    write_sequence(output, steps, |output, step| {
        output.u16(match step {
            ApplicationOperationBodyScopeField::Expect { .. } => 1,
            ApplicationOperationBodyScopeField::Assign { .. } => 2,
            ApplicationOperationBodyScopeField::AssignInput { .. } => 3,
            ApplicationOperationBodyScopeField::ExpectInput { .. } => 9,
            ApplicationOperationBodyScopeField::AssignValue { .. } => 8,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. } => 4,
            ApplicationOperationBodyScopeField::ClearOptional { .. } => 5,
            ApplicationOperationBodyScopeField::RequirePresent { .. } => 6,
            ApplicationOperationBodyScopeField::AssignOptionalInput { .. } => 7,
        })?;
        output.text(step.field().entity())?;
        output.text(step.field().aspect())?;
        output.text(step.field().field())?;
        output.text(step.binding().as_str())?;
        match step {
            ApplicationOperationBodyScopeField::AssignValue { value, .. } => output.text(value),
            ApplicationOperationBodyScopeField::Expect { value, .. }
            | ApplicationOperationBodyScopeField::Assign { value, .. } => {
                super::super::foundational_value::write_aspect_value(output, value)
            }
            ApplicationOperationBodyScopeField::ExpectInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::AssignInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::SetOptionalInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::AssignOptionalInput {
                member,
                scalar_family,
                ..
            } => {
                super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                output.text(member)
            }
            ApplicationOperationBodyScopeField::ClearOptional { scalar_family, .. }
            | ApplicationOperationBodyScopeField::RequirePresent { scalar_family, .. } => {
                super::super::foundational_aspect::write_scalar_type(output, *scalar_family)
            }
        }
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<ApplicationOperationBodyScopeField>, Denial> {
    decode_sequence(input, budget, 20, |input, _| {
        let tag = input.u16()?;
        if !matches!(tag, 1 | 2)
            && !(version >= 2 && tag == 3)
            && !(version >= 5 && matches!(tag, 4 | 5))
            && !(version >= 6 && tag == 6)
            && !(version >= 9 && tag == 7)
            && !(version >= 28 && tag == 8)
            && !(version >= 30 && tag == 9)
        {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
        if tag == 8 {
            return Ok(ApplicationOperationBodyScopeField::AssignValue {
                field,
                binding,
                value: input.text()?.to_owned(),
            });
        }
        if matches!(tag, 3 | 4 | 7 | 9) {
            let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
            let member = input.text()?.to_owned();
            return Ok(if tag == 9 {
                ApplicationOperationBodyScopeField::ExpectInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            } else if tag == 3 {
                ApplicationOperationBodyScopeField::AssignInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            } else if tag == 4 {
                ApplicationOperationBodyScopeField::SetOptionalInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            } else {
                ApplicationOperationBodyScopeField::AssignOptionalInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            });
        }
        if matches!(tag, 5 | 6) {
            let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
            return Ok(if tag == 5 {
                ApplicationOperationBodyScopeField::ClearOptional {
                    field,
                    binding,
                    scalar_family,
                }
            } else {
                ApplicationOperationBodyScopeField::RequirePresent {
                    field,
                    binding,
                    scalar_family,
                }
            });
        }
        let value = super::super::foundational_value::decode_aspect_value(input)?;
        Ok(match tag {
            1 => ApplicationOperationBodyScopeField::Expect {
                field,
                binding,
                value,
            },
            2 => ApplicationOperationBodyScopeField::Assign {
                field,
                binding,
                value,
            },
            _ => unreachable!("tag admitted before operands"),
        })
    })
}
