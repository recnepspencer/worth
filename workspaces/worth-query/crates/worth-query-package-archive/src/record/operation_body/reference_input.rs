//! Existing-reference correspondence has its own v37 sequence, not an entity-ID transport.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyReferenceInput as Reference;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    references: &[Reference],
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
        super::super::foundational_aspect::write_scalar_type(output, reference.scalar_family)
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
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
        Ok(Reference {
            relation,
            from,
            to,
            field,
            binding,
            member,
            key,
            scalar_family,
        })
    })
}
