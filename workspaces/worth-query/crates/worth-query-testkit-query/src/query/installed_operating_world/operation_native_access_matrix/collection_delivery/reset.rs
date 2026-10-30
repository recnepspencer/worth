pub use worth_query::facade::{domain, runtime};

pub use super::super::collection_window::{bound_collection, first_window};
pub use super::super::fixture::{insert_matrix_value, matrix_workspace_without_collection_lookup};
pub use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
pub use super::managed_collection_lease;

pub fn assert_unsupported_reset(patch: &domain::WorthQueryCollectionPatch) {
    assert!(matches!(
        patch.operations(),
        [domain::WorthQueryCollectionPatchOperation::ResetRequired {
            reason: domain::WorthQueryCollectionResetReason::UnsupportedIncrementalMeaning,
            cost: domain::WorthQueryCollectionResetCost {
                fresh_execution_required: true,
                maximum_replacement_rows: 1,
            },
        }]
    ));
}
