use crate::program::WorthQueryDerivedView;
use crate::runtime::tests::support::stateful_bridge_task_runtime as build_stateful_bridge_task_runtime;
use crate::runtime::{
    WorthQueryDerivedPatch, WorthQueryDerivedPatchPayload, WorthQueryDerivedViewMaintainer,
    WorthQueryDerivedViewMaterialization, WorthQueryLiveView, WorthQueryRetainedFieldPath,
    WorthQueryRuntime, WorthQueryWorkspace,
};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

pub(super) struct CertificationTitleListMaintainer;

impl WorthQueryDerivedViewMaintainer for CertificationTitleListMaintainer {
    fn maintain(
        &mut self,
        view: &WorthQueryDerivedView,
        delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let retained_scalar = (
            retained_field_path("value"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                delta
                    .entity_identity
                    .terminal_projection_for_reporting()
                    .to_string(),
            ),
        );
        materialization
            .push_retained_scalar_row([retained_scalar.clone()])
            .expect("certification title row should admit native scalar values");
        WorthQueryDerivedPatch::incremental(
            view.name(),
            crate::memory_workspace::admit_external_commit_label("aspect-api-cert-derived-commit"),
            delta.entity_identity.clone(),
            if view.produced_aspect_touches().is_empty() {
                delta.admitted_touched_aspects().to_vec()
            } else {
                view.produced_aspect_touches().to_vec()
            },
            WorthQueryDerivedPatchPayload::from_retained_scalar_values([retained_scalar])
                .expect("certification title payload should admit native scalar values"),
        )
    }
}

fn retained_field_path(path: &str) -> WorthQueryRetainedFieldPath {
    let fields = path
        .split('.')
        .map(|segment| FieldKey::new(segment.to_string()))
        .collect::<Option<Vec<_>>>()
        .expect("certification retained field path should admit");
    let path = CanonicalFieldPath::new(fields)
        .expect("certification retained field path should not be empty");
    WorthQueryRetainedFieldPath::from_canonical_field_path(path)
}

pub(super) fn stateful_bridge_task_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    build_stateful_bridge_task_runtime(request_execution)
}

pub(super) fn task_live_view<T>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    name: &str,
) -> WorthQueryLiveView<T> {
    workspace
        .live_view(request_execution, name, |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("aspect-api-cert-task")
        })
        .expect("live view should declare")
}
