//! Portable created-field opcodes; decoded values remain descriptive only.
use super::super::sequence::{decode_sequence, write_sequence};
use super::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreation, BinaryEncodingSink, BinaryInput, Denial, Kind,
    RecordDecodeAttempt, WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    creation: &ApplicationOperationBodyCreation,
) -> Result<(), Denial> {
    output.text(creation.entity())?;
    output.text(creation.key())?;
    write_sequence(output, creation.fields(), |output, step| {
        output.u16(match step {
            ApplicationOperationBodyCreatedField::Literal { .. } => 1,
            ApplicationOperationBodyCreatedField::Input { .. } => 2,
            ApplicationOperationBodyCreatedField::ScopeField { .. } => 3,
            ApplicationOperationBodyCreatedField::OptionalInput { .. } => 4,
            ApplicationOperationBodyCreatedField::PrincipalIdentity { .. } => 5,
            ApplicationOperationBodyCreatedField::PresentScopeField { .. } => 6,
            ApplicationOperationBodyCreatedField::SigningProvenance { .. } => 7,
            ApplicationOperationBodyCreatedField::CapturedScopeField { .. } => 8,
            ApplicationOperationBodyCreatedField::RelatedField { .. } => 9,
        })?;
        output.text(step.field().entity())?;
        output.text(step.field().aspect())?;
        output.text(step.field().field())?;
        output.text(step.binding().as_str())?;
        match step {
            ApplicationOperationBodyCreatedField::Literal { value, .. } => {
                super::super::foundational_value::write_aspect_value(output, value)
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
                super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                output.text(member)
            }
            ApplicationOperationBodyCreatedField::ScopeField {
                source,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyCreatedField::PresentScopeField {
                source,
                scalar_family,
                ..
            } => {
                super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
                output.text(source.entity())?;
                output.text(source.aspect())?;
                output.text(source.field())
            }
            ApplicationOperationBodyCreatedField::PrincipalIdentity { .. } => {
                super::principal::write(output, step)
            }
            ApplicationOperationBodyCreatedField::SigningProvenance { .. } => Ok(()),
            ApplicationOperationBodyCreatedField::CapturedScopeField { .. } => {
                super::classified_capture::write(output, step)
            }
            ApplicationOperationBodyCreatedField::RelatedField { .. } => {
                super::related_source::write(output, step)
            }
        }
    })?;
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
    has_creation: bool,
) -> Result<Option<ApplicationOperationBodyCreation>, Denial> {
    if has_creation {
        let entity = input.text()?.to_owned();
        let key = input.text()?.to_owned();
        let fields = decode_sequence(input, budget, 20, |input, _| {
            let tag = input.u16()?;
            if !matches!(tag, 1 | 2)
                && !(version >= 4 && tag == 3)
                && !(version >= 9 && tag == 4)
                && !(version >= 12 && tag == 5)
                && !(version >= 13 && matches!(tag, 6 | 7))
                && !(version >= 14 && tag == 8)
                && !(version >= 20 && tag == 9)
            {
                return Err(Denial::new(Kind::UnsupportedRecordVariant));
            }
            let field =
                ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
            let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
            Ok(match tag {
                1 => ApplicationOperationBodyCreatedField::Literal {
                    field,
                    binding,
                    value: super::super::foundational_value::decode_aspect_value(input)?,
                },
                2 => ApplicationOperationBodyCreatedField::Input {
                    field,
                    binding,
                    scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
                    member: input.text()?.to_owned(),
                },
                3 => ApplicationOperationBodyCreatedField::ScopeField {
                    field,
                    binding,
                    scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
                    source: ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    ),
                },
                4 => ApplicationOperationBodyCreatedField::OptionalInput {
                    field,
                    binding,
                    scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
                    member: input.text()?.to_owned(),
                },
                5 => super::principal::decode(input, field, binding)?,
                6 => ApplicationOperationBodyCreatedField::PresentScopeField {
                    field,
                    binding,
                    scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
                    source: ApplicationFieldBindingLocus::new(
                        input.text()?,
                        input.text()?,
                        input.text()?,
                    ),
                },
                7 => ApplicationOperationBodyCreatedField::SigningProvenance { field, binding },
                8 => super::classified_capture::decode(input, field, binding)?,
                9 => super::related_source::decode(input, field, binding)?,
                _ => unreachable!("creation opcode admitted before operands"),
            })
        })?;
        Ok(Some(ApplicationOperationBodyCreation::from_untrusted(
            entity, key, fields,
        )))
    } else {
        Ok(None)
    }
}
