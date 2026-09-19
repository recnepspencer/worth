mod branch_locality;
mod canonical_keys;
mod conflict_detection;
mod entity_validation;
mod intent_validation;
mod record_lookup;
mod relation_validation;
mod schema_conflicts;

pub(crate) use branch_locality::validate_branch_locality;
pub(crate) use canonical_keys::canonical_intent_key;
pub(crate) use conflict_detection::detect_conflicting_updates;
pub(crate) use intent_validation::{
    collect_created_entity_refs, collect_deleted_relation_ids, validate_intent,
};
pub(crate) use record_lookup::{entity_exists_in_state, relation_exists_in_state};
