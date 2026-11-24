pub mod entity_kinds;
pub mod violations;

pub(crate) use entity_kinds::{contract_candidate_kind_matches, entity_reference_kind};
pub(crate) use violations::{
    canonicalize_violations, relation_violation, storage_inconsistency_violation,
    StorageInconsistencyContext,
};
