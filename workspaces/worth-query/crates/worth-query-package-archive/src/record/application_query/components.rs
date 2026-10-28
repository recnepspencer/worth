//! Portable descriptive selection; fresh declaration/schema admission grants no runtime read.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryComponentMeaning, ApplicationQueryComponentSelection,
    WorthQueryPortableApplicationQueryComponentParts,
};
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    selection: &ApplicationQueryComponentMeaning,
) -> Result<(), Denial> {
    write_type_identity(output, &selection.query_identity())?;
    write_type_identity(output, &selection.slot_identity())?;
    output.text(selection.set())?;
    output.text(selection.member())?;
    output.text(selection.output_name())?;
    output.u16(match selection.selection() {
        ApplicationQueryComponentSelection::OfScopeMember => 1,
        ApplicationQueryComponentSelection::Census => 2,
    })
}
pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryComponentMeaning, Denial> {
    let query = decode_type_identity(input)?;
    let slot = decode_type_identity(input)?;
    let set = input.text()?.to_owned();
    let member = input.text()?.to_owned();
    let output_name = input.text()?.to_owned();
    let selection = match input.u16()? {
        1 => ApplicationQueryComponentSelection::OfScopeMember,
        2 => ApplicationQueryComponentSelection::Census,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationQueryComponentMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryComponentParts {
            query,
            slot,
            set,
            member,
            output_name,
            selection,
        },
    ))
}
