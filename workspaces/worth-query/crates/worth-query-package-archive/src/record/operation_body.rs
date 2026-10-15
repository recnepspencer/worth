//! Versioned portable operation-body carriage. Decoding grants no authority.
use super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};

mod requirements;
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
    let scoped = body.explicit_scope_entity().is_some() || body.is_creation_only();
    let version = if body.has_optional_inputs() {
        9
    } else if scoped {
        8
    } else if !body.requirements().is_empty() {
        7
    } else if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        )
    }) {
        6
    } else if body.scope_fields().iter().any(|step| {
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
    };
    output.u16(version)?;
    if version >= 8 {
        output.text(
            body.scope_entity()
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
        )?;
    }
    write_sequence(output, body.scope_fields(), |output, step| {
        output.u16(match step {
            ApplicationOperationBodyScopeField::Expect { .. } => 1,
            ApplicationOperationBodyScopeField::Assign { .. } => 2,
            ApplicationOperationBodyScopeField::AssignInput { .. } => 3,
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
            }
            | ApplicationOperationBodyScopeField::AssignOptionalInput {
                member,
                scalar_family,
                ..
            } => {
                super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                output.text(member)
            }
            ApplicationOperationBodyScopeField::ClearOptional { scalar_family, .. }
            | ApplicationOperationBodyScopeField::RequirePresent { scalar_family, .. } => {
                super::foundational_aspect::write_scalar_type(output, *scalar_family)
            }
        }
    })?;
    if version >= 5 {
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
                ApplicationOperationBodyCreatedField::OptionalInput { .. } => 4,
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
                }
                | ApplicationOperationBodyCreatedField::OptionalInput {
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
    if version >= 7 {
        requirements::write(output, body.requirements())?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=9) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let scope = if version >= 8 {
        Some(input.text()?.to_owned())
    } else {
        None
    };
    let steps = decode_sequence(input, budget, 20, |input, _| {
        let tag = input.u16()?;
        if !matches!(tag, 1 | 2)
            && !(version >= 2 && tag == 3)
            && !(version >= 5 && matches!(tag, 4 | 5))
            && !(version >= 6 && tag == 6)
            && !(version >= 9 && tag == 7)
        {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
        if matches!(tag, 3 | 4 | 7) {
            let scalar_family = super::foundational_aspect::decode_scalar_type(input)?;
            let member = input.text()?.to_owned();
            return Ok(if tag == 3 {
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
            let scalar_family = super::foundational_aspect::decode_scalar_type(input)?;
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
    let has_creation = if version >= 5 {
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
            if !matches!(tag, 1 | 2) && !(version >= 4 && tag == 3) && !(version >= 9 && tag == 4) {
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
                4 => ApplicationOperationBodyCreatedField::OptionalInput {
                    field,
                    binding,
                    scalar_family: super::foundational_aspect::decode_scalar_type(input)?,
                    member: input.text()?.to_owned(),
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
    let requirements = if version >= 7 {
        requirements::decode(input, budget)?
    } else {
        Vec::new()
    };
    if let Some(scope) = scope {
        Ok(ApplicationOperationBodyProgram::from_untrusted_in_scope(
            scope,
            steps,
            creation,
            requirements,
        ))
    } else {
        let body = ApplicationOperationBodyProgram::from_untrusted_with_requirements(
            steps,
            creation,
            requirements,
        );
        if body.is_creation_only() {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        Ok(body)
    }
}
