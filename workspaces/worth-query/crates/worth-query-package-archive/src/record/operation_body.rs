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
        ApplicationFieldBindingLocus, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    body: &ApplicationOperationBodyProgram,
) -> Result<(), Denial> {
    output.u16(
        if body
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
        })?;
        output.text(step.field().entity())?;
        output.text(step.field().aspect())?;
        output.text(step.field().field())?;
        output.text(step.binding().as_str())?;
        if let Some(value) = step.literal_value() {
            super::foundational_value::write_aspect_value(output, value)
        } else {
            super::foundational_aspect::write_scalar_type(output, step.scalar_family())?;
            output.text(step.input_member().expect("input assignment"))
        }
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationOperationBodyProgram, Denial> {
    let version = input.u16()?;
    if !matches!(version, 1 | 2) {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let steps = decode_sequence(input, budget, 20, |input, _| {
        let tag = input.u16()?;
        if !matches!(tag, 1 | 2) && !(version == 2 && tag == 3) {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.to_owned());
        if tag == 3 {
            return Ok(ApplicationOperationBodyScopeField::AssignInput {
                field,
                binding,
                scalar_family: super::foundational_aspect::decode_scalar_type(input)?,
                member: input.text()?.to_owned(),
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
    Ok(ApplicationOperationBodyProgram::from_untrusted_scope_fields(steps))
}
