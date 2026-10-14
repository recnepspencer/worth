//! Versioned portable operation-body carriage. Decoding grants no authority.
use super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};

#[cfg(test)]
mod tests;
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField,
        ApplicationOperationBodyCreation, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    body: &ApplicationOperationBodyProgram,
) -> Result<(), Denial> {
    output.u16(
        if body.scope_fields().iter().any(|step| {
            matches!(
                step,
                ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::ClearOptional { .. }
            )
        }) {
            5
        } else if body.creation().is_some_and(|creation| {
            creation
                .fields()
                .iter()
                .any(|step| step.source_field().is_some())
        }) {
            4
        } else if body.creation().is_some() {
            3
        } else if body
            .scope_fields()
            .iter()
            .any(|step| step.input_member().is_some())
        {
            2
        } else {
            1
        },
    )?;
    write_sequence(output, body.scope_fields(), |output, step| {
        output.u16(match step {
            ApplicationOperationBodyScopeField::Expect { .. } => 1,
            ApplicationOperationBodyScopeField::Assign { .. } => 2,
            ApplicationOperationBodyScopeField::AssignInput { .. } => 3,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. } => 4,
            ApplicationOperationBodyScopeField::ClearOptional { .. } => 5,
        })?;
        output.text(step.field().entity())?;
        output.text(step.field().aspect())?;
        output.text(step.field().field())?;
        output.text(step.binding().as_str())?;
        match step {
            ApplicationOperationBodyScopeField::Expect { value, .. }
            | ApplicationOperationBodyScopeField::Assign { value, .. } => {
                super::foundational_value::write_aspect_value(output, value)
            }
            ApplicationOperationBodyScopeField::AssignInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::SetOptionalInput {
                member,
                scalar_family,
                ..
            } => {
                super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                output.text(member)
            }
            ApplicationOperationBodyScopeField::ClearOptional { scalar_family, .. } => {
                super::foundational_aspect::write_scalar_type(output, *scalar_family)
            }
        }
    })?;
    if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. }
        )
    }) {
        output.u16(u16::from(body.creation().is_some()))?;
    }
    if let Some(creation) = body.creation() {
        output.text(creation.entity())?;
        output.text(creation.key())?;
        write_sequence(output, creation.fields(), |output, step| {
            output.u16(match step {
                ApplicationOperationBodyCreatedField::Literal { .. } => 1,
                ApplicationOperationBodyCreatedField::Input { .. } => 2,
                ApplicationOperationBodyCreatedField::ScopeField { .. } => 3,
            })?;
            output.text(step.field().entity())?;
            output.text(step.field().aspect())?;
            output.text(step.field().field())?;
            output.text(step.binding().as_str())?;
            match step {
                ApplicationOperationBodyCreatedField::Literal { value, .. } => {
                    super::foundational_value::write_aspect_value(output, value)
                }
                ApplicationOperationBodyCreatedField::Input {
                    member,
                    scalar_family,
                    ..
                } => {
                    super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                    output.text(member)
                }
                ApplicationOperationBodyCreatedField::ScopeField {
                    source,
                    scalar_family,
                    ..
                } => {
                    super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                    output.text(source.entity())?;
                    output.text(source.aspect())?;
                    output.text(source.field())
                }
            }
        })?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=5) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let steps = decode_sequence(input, budget, 20, |input, _| {
        let tag = input.u16()?;
        if !matches!(tag, 1 | 2)
            && !(version >= 2 && tag == 3)
            && !(version == 5 && matches!(tag, 4 | 5))
        {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
        if matches!(tag, 3 | 4) {
            let scalar_family = super::foundational_aspect::decode_scalar_type(input)?;
            let member = input.text()?.to_owned();
            return Ok(if tag == 3 {
                ApplicationOperationBodyScopeField::AssignInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            } else {
                ApplicationOperationBodyScopeField::SetOptionalInput {
                    field,
                    binding,
                    scalar_family,
                    member,
                }
            });
        }
        if tag == 5 {
            return Ok(ApplicationOperationBodyScopeField::ClearOptional {
                field,
                binding,
                scalar_family: super::foundational_aspect::decode_scalar_type(input)?,
            });
        }
        let value = super::foundational_value::decode_aspect_value(input)?;
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
    })?;
    let has_creation = if version == 5 {
        match input.u16()? {
            0 => false,
            1 => true,
            _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    } else {
        version >= 3
    };
    let creation = if has_creation {
        let entity = input.text()?.to_owned();
        let key = input.text()?.to_owned();
        let fields = decode_sequence(input, budget, 20, |input, _| {
            let tag = input.u16()?;
            if !matches!(tag, 1 | 2) && !(version >= 4 && tag == 3) {
                return Err(Denial::new(Kind::UnsupportedRecordVariant));
            }
            let field =
                ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
            let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
            Ok(match tag {
                1 => ApplicationOperationBodyCreatedField::Literal {
                    field,
                    binding,
                    value: super::foundational_value::decode_aspect_value(input)?,
                },
                2 => ApplicationOperationBodyCreatedField::Input {
                    field,
                    binding,
                    scalar_family: super::foundational_aspect::decode_scalar_type(input)?,
                    member: input.text()?.to_owned(),
                },
                3 => ApplicationOperationBodyCreatedField::ScopeField {
                    field,
                    binding,
                    scalar_family: super::foundational_aspect::decode_scalar_type(input)?,
                    source: ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    ),
                },
                _ => unreachable!("creation opcode admitted before operands"),
            })
        })?;
        Some(ApplicationOperationBodyCreation::from_untrusted(
            entity, key, fields,
        ))
    } else {
        None
    };
    Ok(ApplicationOperationBodyProgram::from_untrusted_with_creation(steps, creation))
}
