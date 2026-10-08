//! Callback over the selected exact-root value, with no scalar/carrier copy.
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime},
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
};

pub(in crate::domain_computation::primary_graph) fn observe_field_value_borrowed<Output>(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(Option<&'raw AspectValue>) -> Output,
) -> Option<Output> {
    let field = locator.field_path().fields().first()?.clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    let mut action = Some(action);
    runtime
        .read_truth()
        .project_snapshot(snapshot)?
        .entity_record_with_projection_scope(entity_id, scope, |record| {
            if record.kind_id() != kind || record.lifecycle() != RecordLifecycleState::Live {
                return None;
            }
            let action = action
                .take()
                .expect("single-entity projection invokes once");
            Some(action(
                record.aspect_field_value(locator.aspect().aspect_key(), &field),
            ))
        })
}
