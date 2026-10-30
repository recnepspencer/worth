pub use worth_query::facade::{domain, runtime};

pub use super::super::collection_window::{bound_collection, first_window};
pub use super::super::fixture::{insert_matrix_value, matrix_workspace};
pub use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
pub use super::{managed_collection_lease, required_patch};

pub fn update_order(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    order: &str,
) {
    workspace
        .update(request_execution, entity, |mutation| {
            mutation.set_aspect(
                runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                matrix_value_with_order(0, order),
            )
        })
        .unwrap();
}
