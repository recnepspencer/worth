use crate::symbols::data::ClientKey;

pub(super) fn merge_client_key(
    prefix: &str,
    record: &crate::transactions::data::primitives::RecordRef,
) -> ClientKey {
    let suffix = match record {
        crate::transactions::data::primitives::RecordRef::Entity(entity_id) => format!(
            "entity-{}-{}-{}",
            entity_id.partition_id.0, entity_id.local_slot.0, entity_id.generation.0
        ),
        crate::transactions::data::primitives::RecordRef::Relation(relation_id) => format!(
            "relation-{}-{}-{}",
            relation_id.partition_id.0, relation_id.local_slot.0, relation_id.generation.0
        ),
    };
    ClientKey::raw(format!("{prefix}-{suffix}"))
}
