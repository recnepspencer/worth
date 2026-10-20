//! Version-twelve actor initializer carriage, never native principal authority.
use super::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyCreatedField, BinaryEncodingSink,
    BinaryInput, Denial, Kind, WorthQueryPortableTypeIdentity,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    step: &ApplicationOperationBodyCreatedField,
) -> Result<(), Denial> {
    let ApplicationOperationBodyCreatedField::PrincipalIdentity {
        source,
        scalar_family,
        mutation_binding,
        principal_binding,
        ..
    } = step
    else {
        return Err(Denial::new(Kind::InvalidRecordShape));
    };
    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
    output.text(mutation_binding)?;
    output.text(principal_binding)?;
    output.text(source.entity())?;
    output.text(source.aspect())?;
    output.text(source.field())
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    field: ApplicationFieldBindingLocus,
    binding: WorthQueryPortableTypeIdentity,
) -> Result<ApplicationOperationBodyCreatedField, Denial> {
    Ok(ApplicationOperationBodyCreatedField::PrincipalIdentity {
        field,
        binding,
        scalar_family: super::super::foundational_aspect::decode_scalar_type(input)?,
        mutation_binding: input.text()?.to_owned(),
        principal_binding: input.text()?.to_owned(),
        source: ApplicationFieldBindingLocus::new(input.text()?, input.text()?, input.text()?),
    })
}
