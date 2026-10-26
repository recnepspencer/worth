//! Whole-orbit query meaning is decoded as untrusted declaration data.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrbitMeaning, WorthQueryPortableApplicationQueryOrbitParts,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    orbit: &ApplicationQueryOrbitMeaning,
) -> Result<(), Denial> {
    write_type_identity(output, &orbit.query_identity())?;
    write_type_identity(output, &orbit.slot_identity())?;
    output.text(orbit.relation())?;
    output.text(orbit.from())?;
    output.text(orbit.to())?;
    output.text(orbit.output_name())
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryOrbitMeaning, Denial> {
    Ok(ApplicationQueryOrbitMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryOrbitParts {
            query: decode_type_identity(input)?,
            slot: decode_type_identity(input)?,
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            output_name: input.text()?.to_owned(),
        },
    ))
}
