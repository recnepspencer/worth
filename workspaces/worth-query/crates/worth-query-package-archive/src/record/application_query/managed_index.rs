//! Committed-index selector payload; version is checked before field allocation.
use super::*;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryManagedIndexMeaning as Meaning,
    WorthQueryPortableQueryManagedIndexParts as Parts,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, meaning: &Meaning) -> Result<(), Denial> {
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let parts = meaning.parts();
    output.u16(parts.semantic_version)?;
    output.text(&parts.index)?;
    output.text(&parts.probe_parameter)
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Meaning, Denial> {
    let semantic_version = input.u16()?;
    if semantic_version != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let meaning = Meaning::from_untrusted_parts(Parts {
        semantic_version,
        index: input.text()?.to_owned(),
        probe_parameter: input.text()?.to_owned(),
    });
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(meaning)
}
