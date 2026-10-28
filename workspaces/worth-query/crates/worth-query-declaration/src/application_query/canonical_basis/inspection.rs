use super::{
    disclosure::append_disclosure,
    entry::{text, unsigned_u64},
    prepare_artifact, ApplicationQueryCanonicalArtifact,
};
use crate::application_query::WorthQueryPortableApplicationQueryInspectionParts;

pub(in crate::application_query) fn prepare_inspection_basis(
    parts: &WorthQueryPortableApplicationQueryInspectionParts,
) -> ApplicationQueryCanonicalArtifact {
    let mut entries = vec![
        text("meaning.kind", "query-inspection-metadata"),
        unsigned_u64("meaning.version", u64::from(parts.semantic_version)),
    ];
    for (name, value) in parts.binding.canonical_fields() {
        entries.push(text(format!("inspection.binding.{name}"), value));
    }
    append_disclosure(&mut entries, &parts.metadata_disclosure);
    prepare_artifact(entries)
}
