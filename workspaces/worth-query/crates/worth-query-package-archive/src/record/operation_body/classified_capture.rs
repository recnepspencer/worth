//! Body v14 retains explicit capture role and source presence; older encodings remain frozen.
use super::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCreatedField as Step,
    BinaryEncodingSink, BinaryInput, Denial, Kind, WorthQueryPortableTypeIdentity,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyCaptureRole as Role,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, step: &Step) -> Result<(), Denial> {
    let Step::CapturedScopeField {
        source,
        scalar_family,
        source_presence,
        role,
        ..
    } = step
    else {
        unreachable!("classified capture opcode dispatch");
    };
    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
    output.text(source.entity())?;
    output.text(source.aspect())?;
    output.text(source.field())?;
    output.u16(match source_presence {
        Presence::Required => 1,
        Presence::Optional => 2,
    })?;
    output.u16(match role {
        Role::Subject => 1,
        Role::AuditContext => 2,
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    field: Locus,
    binding: WorthQueryPortableTypeIdentity,
) -> Result<Step, Denial> {
    let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
    let source = Locus::new(input.text()?, input.text()?, input.text()?);
    let source_presence = match input.u16()? {
        1 => Presence::Required,
        2 => Presence::Optional,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let role = match input.u16()? {
        1 => Role::Subject,
        2 => Role::AuditContext,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(Step::CapturedScopeField {
        field,
        binding,
        scalar_family,
        source,
        source_presence,
        role,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_capture_role_or_source_presence_never_becomes_a_default() {
        let field = Locus::new("Evidence", "Facts", "Text");
        let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into());
        let step = Step::CapturedScopeField {
            field: field.clone(),
            binding: binding.clone(),
            scalar_family: worth_foundational::facade::ScalarAspectType::String,
            source: Locus::new("Draft", "Facts", "Text"),
            source_presence: Presence::Optional,
            role: Role::AuditContext,
        };
        let mut output = crate::binary_output::BinaryOutput::with_capacity(128);
        write(&mut output, &step).unwrap();
        let bytes = output.into_bytes();
        for trailing_offset in [2, 4] {
            let mut corrupt = bytes.clone();
            let start = corrupt.len() - trailing_offset;
            corrupt[start..start + 2].copy_from_slice(&99_u16.to_be_bytes());
            assert_eq!(
                decode(
                    &mut BinaryInput::new(&corrupt),
                    field.clone(),
                    binding.clone()
                )
                .unwrap_err()
                .kind(),
                Kind::UnsupportedRecordVariant
            );
        }
    }
}
