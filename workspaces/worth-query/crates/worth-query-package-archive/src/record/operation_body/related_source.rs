//! Related-field opcode 9 is admitted only by body profile 20 or newer.
use super::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField as Step, BinaryEncodingSink,
    BinaryInput, Denial, WorthQueryPortableTypeIdentity,
};
use crate::record::application_schema::wire_vocabulary::{decode_presence, write_presence};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRelatedFieldSource as Source;
pub(super) fn write(output: &mut dyn BinaryEncodingSink, step: &Step) -> Result<(), Denial> {
    let Step::RelatedField {
        source,
        scalar_family,
        ..
    } = step
    else {
        unreachable!("related field dispatcher");
    };
    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
    for value in [
        &source.relation,
        &source.from,
        &source.to,
        source.field.entity(),
        source.field.aspect(),
        source.field.field(),
    ] {
        output.text(value)?;
    }
    write_presence(output, source.presence)
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    field: ApplicationFieldBindingLocus,
    binding: WorthQueryPortableTypeIdentity,
) -> Result<Step, Denial> {
    let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    let source_field =
        ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?);
    let presence = decode_presence(input)?;
    Ok(Step::RelatedField {
        field,
        binding,
        scalar_family,
        source: Source {
            relation,
            from,
            to,
            field: source_field,
            presence,
        },
    })
}
