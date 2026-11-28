mod branch_locality;
mod conflict_detection;
mod entity_validation;
mod intent_validation;
mod record_lookup;
mod relation_validation;
mod schema_conflicts;

pub use branch_locality::validate_branch_locality;
pub use conflict_detection::detect_conflicting_updates;
pub use intent_validation::collect_created_entity_refs;
pub use intent_validation::collect_deleted_relation_ids;
pub use intent_validation::validate_intent;
pub use record_lookup::{entity_exists_in_state, relation_exists_in_state};
