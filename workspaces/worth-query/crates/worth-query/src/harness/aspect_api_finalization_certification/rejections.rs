use crate::runtime::{WorthQueryIntentDeclaration, WorthQueryIntentInput, WorthQueryRuntimeError};

use super::fixture::stateful_bridge_task_runtime;
use super::{
    title_value_touch, AspectApiFinalizationFailureClass, AspectApiFinalizationRejectionBundle,
};

pub(super) fn unsupported_intent_rejection(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> AspectApiFinalizationRejectionBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.intent-denial")
        .expect("workspace should open");
    let report = workspace.public_mutation_surface_report();
    let closeout = workspace.public_aspect_api_finalization_closeout();
    let error = workspace
        .intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "unsupported-intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                intent_input([("entity", "task-1")]),
            ),
        )
        .expect_err("unsupported runtime should deny intent typed and early");

    match error {
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
            AspectApiFinalizationRejectionBundle {
                failure_class: AspectApiFinalizationFailureClass::SupportDenied,
                failure_kind: denial.family().to_string(),
                failure_digest: crate::harness::certification::digest_parts(&[
                    denial.family().to_string(),
                    denial.reason().to_string(),
                ]),
                support_matrix_digest: workspace
                    .public_support_matrix()
                    .matrix_digest()
                    .terminal_projection_for_reporting()
                    .to_string(),
                mutation_surface_report_digest: report.report_digest().to_string(),
                closeout_digest: closeout.closeout_digest().to_string(),
            }
        }
        other => panic!("expected typed support denial, got {other:?}"),
    }
}

pub(super) fn duplicate_aspect_authoring_rejection(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> AspectApiFinalizationRejectionBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.duplicate-denial")
        .expect("workspace should open");
    let report = workspace.public_mutation_surface_report();
    let closeout = workspace.public_aspect_api_finalization_closeout();
    let error = workspace
        .update(
            request_execution,
            crate::memory_workspace::admit_authored_entity_label("entity:1:1:1"),
            |task| {
                task.clear(title_value_touch())
                    .set_aspect(title_value_touch(), string_aspect_value("Buy milk"))
            },
        )
        .expect_err("duplicate aspect authoring should fail closed");

    match error {
        WorthQueryRuntimeError::Workspace(error) => AspectApiFinalizationRejectionBundle {
            failure_class: AspectApiFinalizationFailureClass::AuthoringDenied,
            failure_kind: "workspace-authoring".to_string(),
            failure_digest: crate::harness::certification::digest_parts(&[error.to_string()]),
            support_matrix_digest: workspace
                .public_support_matrix()
                .matrix_digest()
                .terminal_projection_for_reporting()
                .to_string(),
            mutation_surface_report_digest: report.report_digest().to_string(),
            closeout_digest: closeout.closeout_digest().to_string(),
        },
        other => panic!("expected workspace authoring denial, got {other:?}"),
    }
}

fn string_aspect_value(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn intent_input(
    fields: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> WorthQueryIntentInput {
    WorthQueryIntentInput::object(
        fields
            .into_iter()
            .map(|(field, value)| (field, WorthQueryIntentInput::string(value))),
    )
}
