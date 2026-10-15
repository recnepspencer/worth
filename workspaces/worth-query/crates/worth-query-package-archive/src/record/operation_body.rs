//! Versioned portable operation-body carriage. Decoding grants no authority.
use super::{
    decode_budget::RecordDecodeAttempt,
    sequence::{decode_sequence, write_sequence},
};

mod creation_fields;
mod creation_links;
mod outputs;
mod principal;
mod requirements;
mod signing;
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
    let version = if signing::uses_signed_sources(body) {
        13
    } else if body.creation().is_some_and(|creation| {
        creation
            .fields()
            .iter()
            .any(|step| step.principal_source().is_some())
    }) {
        12
    } else if body
        .creation()
        .is_some_and(|creation| !creation.links().is_empty())
    {
        11
    } else if !body.outputs().is_empty() {
        10
    } else if body.has_optional_inputs() {
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
        creation_fields::write(output, creation)?;
    }
    if version >= 7 {
        requirements::write(output, body.requirements())?;
    }
    if version >= 10 {
        outputs::write(output, body.outputs())?;
    }
    if version >= 11 {
        creation_links::write(
            output,
            body.creation().map_or(&[], |creation| creation.links()),
        )?;
    }
    if version >= 13 {
        signing::write(
            output,
            body.creation().and_then(|creation| creation.signing()),
        )?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1..=13) {
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
    let creation = creation_fields::decode(input, budget, version, has_creation)?;
    let requirements = if version >= 7 {
        requirements::decode(input, budget, version)?
    } else {
        Vec::new()
    };
    let outputs = if version >= 10 {
        outputs::decode(input, budget)?
    } else {
        Vec::new()
    };
    let links = if version >= 11 {
        creation_links::decode(input, budget)?
    } else {
        Vec::new()
    };
    let signing = if version >= 13 {
        signing::decode(input)?
    } else {
        None
    };
    let creation = match creation {
        Some(creation) => Some(
            creation
                .with_untrusted_links(links)
                .with_untrusted_signing(signing),
        ),
        None if links.is_empty() && signing.is_none() => None,
        None => return Err(Denial::new(Kind::InvalidRecordShape)),
    };
    if let Some(scope) = scope {
        Ok(ApplicationOperationBodyProgram::from_untrusted_in_scope(
            scope,
            steps,
            creation,
            requirements,
        )
        .with_untrusted_outputs(outputs))
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
