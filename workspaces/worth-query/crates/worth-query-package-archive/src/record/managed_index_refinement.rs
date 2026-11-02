//! One wire owner for the versioned row-local domain meaning in Index/Parent.
use crate::{
    binary_encoding::BinaryEncodingSink,
    binary_input::BinaryInput,
    denial::{
        WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
    },
};
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexRefinementMeaning as Meaning,
    WorthQueryPortableManagedIndexRefinementParts as Parts,
};
pub(super) fn write(output: &mut dyn BinaryEncodingSink, meaning: &Meaning) -> Result<(), Denial> {
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let p = meaning.parts();
    output.u16(p.semantic_version)?;
    output.text(&p.identity)?;
    output.u32(p.version)
}
pub(super) fn decode(input: &mut BinaryInput<'_>) -> Result<Meaning, Denial> {
    let semantic_version = input.u16()?;
    if semantic_version != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    let meaning = Meaning::from_untrusted_parts(Parts {
        semantic_version,
        identity: input.text()?.to_owned(),
        version: input.u32()?,
    });
    meaning
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    Ok(meaning)
}
