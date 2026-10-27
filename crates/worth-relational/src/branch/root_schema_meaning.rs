use crate::schema::data::RelationalSchemaRegistry;

pub(super) fn registry_meaning_matches(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
) -> bool {
    if source.derived_parts() != target.derived_parts()
        || source.entity_kinds.len() != target.entity_kinds.len()
        || source.relation_kinds.len() != target.relation_kinds.len()
    {
        return false;
    }
    let entities_match = source.entity_kinds.iter().all(|(kind_id, source_kind)| {
        target.entity_kinds.get(kind_id).is_some_and(|target_kind| {
            let mut normalized = target_kind.clone();
            normalized.schema_id = source_kind.schema_id.clone();
            normalized.schema_version_id = source_kind.schema_version_id;
            normalized == *source_kind
        })
    });
    let relations_match = source.relation_kinds.iter().all(|(kind_id, source_kind)| {
        target
            .relation_kinds
            .get(kind_id)
            .is_some_and(|target_kind| {
                let mut normalized = target_kind.clone();
                normalized.schema_id = source_kind.schema_id.clone();
                normalized.schema_version_id = source_kind.schema_version_id;
                normalized == *source_kind
            })
    });
    entities_match && relations_match
}
