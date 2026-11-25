//! Existing-reference correspondence has its own v37 sequence, not an entity-ID transport.
//! v42 adds the selection mode; older profiles carry only nullable replacement.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyReferenceInput as Reference,
    ApplicationOperationBodyReferenceInputMode as Mode,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    references: &[Reference],
    version: u16,
) -> Result<(), Denial> {
    write_sequence(output, references, |output, reference| {
        for part in [
            &reference.relation,
            &reference.from,
            &reference.to,
            reference.field.entity(),
            reference.field.aspect(),
            reference.field.field(),
            reference.binding.as_str(),
            &reference.member,
            &reference.key,
        ] {
            output.text(part)?;
        }
        super::super::foundational_aspect::write_scalar_type(output, reference.scalar_family)?;
        match (&reference.mode, version >= 42) {
            (Mode::ReplaceNullable, false) => Ok(()),
            (Mode::ReplaceNullable, true) => output.u16(0),
            (Mode::ToggleExact { linked_member }, true) => {
                output.u16(1)?;
                output.text(linked_member)
            }
            (Mode::ToggleExact { .. }, false) => Err(Denial::new(Kind::UnsupportedRecordVariant)),
        }
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    version: u16,
) -> Result<Vec<Reference>, Denial> {
    decode_sequence(input, budget, 20, |input, _| {
        let relation = input.text()?.into();
        let from = input.text()?.into();
        let to = input.text()?.into();
        let field = ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
        let binding = WorthQueryPortableTypeIdentity::from_untrusted(input.text()?.into());
        let member = input.text()?.into();
        let key = input.text()?.into();
        let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
        let mode = if version < 42 {
            Mode::ReplaceNullable
        } else {
            match input.u16()? {
                0 => Mode::ReplaceNullable,
                1 => Mode::ToggleExact {
                    linked_member: input.text()?.into(),
                },
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            }
        };
        Ok(Reference {
            relation,
            from,
            to,
            field,
            binding,
            member,
            key,
            scalar_family,
            mode,
        })
    })
}
