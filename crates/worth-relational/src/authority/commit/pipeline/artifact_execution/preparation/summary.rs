use super::*;

pub(super) fn summarize_commit_aspects(
    deltas: &[crate::authority::mutation::CanonicalRecordAspectDelta],
) -> CommitAspectSummary {
    let mut changed_entity_aspect_count = 0;
    let mut changed_relation_aspect_count = 0;
    let mut touched_aspects = Vec::new();
    let mut opaque_aspect_delta_count = 0;
    let mut zero_aspect_structural_delta_count = 0;

    for delta in deltas {
        let aspect_count = delta.changed_aspects.len();
        match delta.target {
            RecordRef::Entity(_) => changed_entity_aspect_count += aspect_count,
            RecordRef::Relation(_) => changed_relation_aspect_count += aspect_count,
        }
        touched_aspects.extend(delta.changed_aspects.iter().cloned());
        if delta.contains_opaque_aspect {
            opaque_aspect_delta_count += 1;
        }
        if delta.changed_aspects.is_empty() {
            zero_aspect_structural_delta_count += 1;
        }
    }

    CommitAspectSummary {
        changed_entity_aspect_count,
        changed_relation_aspect_count,
        touched_aspects: crate::publication::patch::data::ordered_aspect_keys(touched_aspects),
        opaque_aspect_delta_count,
        zero_aspect_structural_delta_count,
    }
}
