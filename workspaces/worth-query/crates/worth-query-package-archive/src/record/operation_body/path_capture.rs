//! V29 uses the existing query-path codec and retains both presence contracts.
use super::{
    ApplicationFieldBindingLocus as Locus, ApplicationOperationBodyCreatedField as Field,
    BinaryEncodingSink, BinaryInput, Denial, Kind, RecordDecodeAttempt,
    WorthQueryPortableTypeIdentity,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence as Presence, ApplicationOperationBodyCaptureRole as Role,
    ApplicationOperationBodyPathFieldSource as Source,
};
pub(super) fn write(output: &mut dyn BinaryEncodingSink, step: &Field) -> Result<(), Denial> {
    let Field::CapturedPathField {
        source,
        scalar_family,
        role,
        target_presence,
        ..
    } = step
    else {
        unreachable!("path capture opcode")
    };
    super::super::foundational_aspect::write_scalar_type(output, *scalar_family)?;
    super::super::application_query::write_root_path(output, &source.path)?;
    output.text(source.field.entity())?;
    output.text(source.field.aspect())?;
    output.text(source.field.field())?;
    write_presence(output, source.presence)?;
    write_presence(output, *target_presence)?;
    output.u16(match role {
        Role::Subject => 1,
        Role::AuditContext => 2,
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    field: Locus,
    binding: WorthQueryPortableTypeIdentity,
) -> Result<Field, Denial> {
    let scalar_family = super::super::foundational_aspect::decode_scalar_type(input)?;
    let path = super::super::application_query::decode_root_path(input, budget)?;
    let source_field = Locus::new(input.text()?, input.text()?, input.text()?);
    let presence = decode_presence(input)?;
    let target_presence = decode_presence(input)?;
    let role = match input.u16()? {
        1 => Role::Subject,
        2 => Role::AuditContext,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let source = Source {
        path,
        field: source_field,
        presence,
    };
    if !source.has_supported_shape() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    Ok(Field::CapturedPathField {
        field,
        binding,
        scalar_family,
        source,
        role,
        target_presence,
    })
}
fn write_presence(output: &mut dyn BinaryEncodingSink, presence: Presence) -> Result<(), Denial> {
    output.u16(match presence {
        Presence::Required => 1,
        Presence::Optional => 2,
    })
}
fn decode_presence(input: &mut BinaryInput<'_>) -> Result<Presence, Denial> {
    match input.u16()? {
        1 => Ok(Presence::Required),
        2 => Ok(Presence::Optional),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
