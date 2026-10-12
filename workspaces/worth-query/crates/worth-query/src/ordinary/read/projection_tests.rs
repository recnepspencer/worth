use crate::authoring::{AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName};
use crate::ordinary::{WorthQueryOutcomeNavigation, WorthQueryOutcomePosture};
use crate::runtime::tests::support::stateful_bridge_task_runtime;
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};

use super::{current, declare, project_facts, WorthQueryProjectionOutcome};

#[test]
fn completed_read_consumes_projection_without_exposing_phase_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("ordinary-read-projection")
        .expect("workspace should open");
    let completion = declare(identity_read)
        .expect("read should declare")
        .using(current())
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("read should complete");

    let outcome = completion.consume_projection(project_facts().entity_identities());

    assert_eq!(
        outcome.posture(),
        WorthQueryOutcomePosture::Completed,
        "projection outcome: {outcome:#?}"
    );
    let authority = outcome.authority().expect("projection should admit");
    assert_eq!(authority.receipt().authority_reopen_count(), 0);
    assert_eq!(authority.consumer_contract().requested_fact_count(), 1);
}

#[test]
fn projection_outcome_preserves_family_specific_payload_accessors() {
    fn inspect(outcome: &WorthQueryProjectionOutcome) {
        match outcome.posture() {
            WorthQueryOutcomePosture::Completed => assert!(outcome.authority().is_some()),
            WorthQueryOutcomePosture::Advisory => assert!(outcome.advisory().is_some()),
            WorthQueryOutcomePosture::Violation => assert!(outcome.violation().is_some()),
            WorthQueryOutcomePosture::Deferred => assert!(outcome.deferred().is_some()),
            WorthQueryOutcomePosture::Unavailable => assert!(outcome.unavailable().is_some()),
        }
    }

    let _ = inspect as fn(&WorthQueryProjectionOutcome);
}

fn identity_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "Task",
        QuerySchemaView::new(
            "ordinary-projection-read",
            [SchemaFieldView::new(
                AspectName::new("identity").expect("aspect should build"),
                FieldName::new("id").expect("field should build"),
                ScalarAspectType::String,
            )],
            [],
        ),
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id").expect("projection should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("result field should build"),
            )
        },
    )
}
