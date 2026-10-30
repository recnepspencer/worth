//! One exclusive root-selection choice. Legacy none/orbit/component bytes stay exact.
use super::super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryComponentMeaning, ApplicationQueryManagedIndexMeaning,
    ApplicationQueryOrbitMeaning,
};

pub(in crate::record::application_query) fn require_exclusive(
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    let count = usize::from(parts.orbit().is_some())
        + usize::from(parts.component_selection().is_some())
        + usize::from(parts.managed_index().is_some());
    if count > 1 || (parts.managed_index().is_some() && !parts.root_paths().is_empty()) {
        Err(Denial::new(Kind::InvalidRecordShape))
    } else {
        Ok(())
    }
}

pub(in crate::record::application_query) fn write(
    output: &mut dyn BinaryEncodingSink,
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    require_exclusive(parts)?;
    if let Some(index) = parts.managed_index() {
        output.u16(3)?;
        return super::super::managed_index::write(output, index);
    }
    match (parts.orbit(), parts.component_selection()) {
        (None, None) => output.u16(0),
        (Some(orbit), None) => {
            output.u16(1)?;
            super::super::orbit::write(output, orbit)
        }
        (None, Some(selection)) => {
            output.u16(2)?;
            super::super::components::write(output, selection)
        }
        (Some(_), Some(_)) => Err(Denial::new(Kind::InvalidRecordShape)),
    }
}

type Selection = (
    Option<ApplicationQueryOrbitMeaning>,
    Option<ApplicationQueryComponentMeaning>,
    Option<ApplicationQueryManagedIndexMeaning>,
);
pub(in crate::record::application_query) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<Selection, Denial> {
    match input.u16()? {
        0 => Ok((None, None, None)),
        1 => super::super::orbit::decode(input).map(|value| (Some(value), None, None)),
        2 => super::super::components::decode(input).map(|value| (None, Some(value), None)),
        3 => super::super::managed_index::decode(input).map(|value| (None, None, Some(value))),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
