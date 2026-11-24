use crate::schema::data::RelationalSchemaRegistry;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub(super) fn registry_meaning_matches(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
) -> bool {
    if (*source.owner_content_formats()).snapshot() != (*target.owner_content_formats()).snapshot()
        || source.derived_parts() != target.derived_parts()
        || source.persistent_name_schemes() != target.persistent_name_schemes()
        || source.kind_name_schemes_map() != target.kind_name_schemes_map()
        || (*source.owner_entity_kinds()).len() != (*target.owner_entity_kinds()).len()
        || (*source.owner_relation_kinds()).len() != (*target.owner_relation_kinds()).len()
    {
        return false;
    }
    let entities_match = (*source.owner_entity_kinds())
        .iter()
        .all(|(kind_id, source_kind)| {
            (*target.owner_entity_kinds())
                .get(kind_id)
                .is_some_and(|target_kind| {
                    let mut normalized = target_kind.clone();
                    normalized.schema_id = source_kind.schema_id.clone();
                    normalized.schema_version_id = source_kind.schema_version_id;
                    normalized == *source_kind
                })
        });
    let relations_match = (*source.owner_relation_kinds())
        .iter()
        .all(|(kind_id, source_kind)| {
            (*target.owner_relation_kinds())
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
