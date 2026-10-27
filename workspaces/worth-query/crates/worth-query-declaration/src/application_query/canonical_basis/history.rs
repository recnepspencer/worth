use super::{
    disclosure::append_disclosure,
    entry::{text, unsigned_u64, unsigned_usize},
    prepare_artifact, ApplicationQueryCanonicalArtifact,
};
use crate::application_query::{
    ApplicationHistoryTarget, WorthQueryPortableApplicationHistoryQueryParts,
};

pub(in crate::application_query) fn prepare_history_basis(
    parts: &WorthQueryPortableApplicationHistoryQueryParts,
) -> ApplicationQueryCanonicalArtifact {
    let mut entries = vec![
        text("meaning.kind", "record-aspect-history-v1"),
        text("history.name", &parts.name),
        text("history.type", parts.history_type.as_str()),
        text("history.order", "exact-product-parent-native-canonical-v1"),
        text("history.subject", "retained-exact-generation-v1"),
        text("history.metadata", "occurrence-ancestry-continuation-v1"),
    ];
    for (name, value) in parts.subject_read.canonical_fields() {
        entries.push(text(format!("subject-read.{name}"), value));
    }
    match &parts.target {
        ApplicationHistoryTarget::Entity { entity } => {
            entries.extend([text("target.kind", "entity"), text("target.entity", entity)])
        }
        ApplicationHistoryTarget::Relation {
            query,
            slot,
            relation,
            from,
            to,
            direction,
            cardinality,
            output_name,
        } => {
            entries.extend([
                text("target.kind", "relation"),
                text("target.query", query.as_str()),
                text("target.slot", slot.as_str()),
                text("target.relation", relation),
                text("target.from", from),
                text("target.to", to),
                text(
                    "target.direction",
                    super::result_shape::traversal_direction_name(*direction),
                ),
                text(
                    "target.cardinality",
                    super::result_shape::cardinality_name(*cardinality),
                ),
                text("target.output", output_name),
            ]);
        }
    }
    entries.push(unsigned_usize("aspect-count", parts.aspects.len()));
    for (index, aspect) in parts.aspects.iter().enumerate() {
        entries.extend([
            text(format!("aspect[{index}].name"), &aspect.aspect),
            unsigned_u64(format!("aspect[{index}].identity"), aspect.identity.0),
            unsigned_u64(format!("aspect[{index}].revision"), aspect.revision.0),
        ]);
    }
    append_disclosure(&mut entries, &parts.metadata_disclosure);
    prepare_artifact(entries)
}
