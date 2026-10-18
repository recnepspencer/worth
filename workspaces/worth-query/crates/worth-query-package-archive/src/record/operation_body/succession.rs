//! Body v21 carries a Scope/Preserve succession declaration, never native authority.
use super::{BinaryEncodingSink, BinaryInput, Denial, Kind};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccession as Succession;
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    succession: Option<&Succession>,
) -> Result<(), Denial> {
    output.u8(u8::from(succession.is_some()))?;
    if let Some(succession) = succession {
        output.text(succession.workflow_spec())?;
        output.text(succession.binding())?;
        output.text(succession.role())?;
    }
    Ok(())
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Option<Succession>, Denial> {
    match input.u8()? {
        0 => Ok(None),
        1 => Ok(Some(Succession::from_untrusted(
            input.text()?.to_owned(),
            input.text()?.to_owned(),
            input.text()?.to_owned(),
        ))),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
