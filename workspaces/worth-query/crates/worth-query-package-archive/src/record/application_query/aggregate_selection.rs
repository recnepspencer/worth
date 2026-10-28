//! Extends the existing optional aggregate slot without reinterpreting tags 0/1.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryComponentMeaning, ApplicationQueryOrbitMeaning,
};
pub(super) fn require_exclusive(
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    if parts.orbit().is_some() && parts.component_selection().is_some() {
        Err(Denial::new(Kind::InvalidRecordShape))
    } else {
        Ok(())
    }
}
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    require_exclusive(parts)?;
    match (parts.orbit(), parts.component_selection()) {
        (None, None) => output.u16(0),
        (Some(orbit), None) => {
            output.u16(1)?;
            super::orbit::write(output, orbit)
        }
        (None, Some(selection)) => {
            output.u16(2)?;
            super::components::write(output, selection)
        }
        (Some(_), Some(_)) => Err(Denial::new(Kind::InvalidRecordShape)),
    }
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<
    (
        Option<ApplicationQueryOrbitMeaning>,
        Option<ApplicationQueryComponentMeaning>,
    ),
    Denial,
> {
    match input.u16()? {
        0 => Ok((None, None)),
        1 => super::orbit::decode(input).map(|orbit| (Some(orbit), None)),
        2 => super::components::decode(input).map(|selection| (None, Some(selection))),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
