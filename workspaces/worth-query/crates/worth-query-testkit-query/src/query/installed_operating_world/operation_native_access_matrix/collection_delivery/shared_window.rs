pub use worth_query::facade::{domain, runtime};

pub use super::super::collection_window::{bound_collection, first_window, settled_with_order_key};
pub use super::super::fixture::{insert_matrix_value, matrix_workspace};
pub use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
pub use super::required_patch;

pub fn shared_leases(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (super::CollectionLease, super::CollectionLease) {
    let (subject, _) = settled_with_order_key(workspace);
    let live = match subject
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("shared collection subject did not promote"),
    };
    let (candidate, _) = settled_with_order_key(workspace);
    let shared = match live.share_with(candidate.into_lifecycle(), workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("shared collection admission stopped: {}", stop.detail())
        }
    };
    shared.into_leases()
}
