use crate::authoring::{AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName};
use crate::basis_lifecycle::basis_lifecycle;
use crate::ordinary::read::{current, declare as declare_read};
use crate::ordinary::{WorthQueryOutcomeNavigation, WorthQueryOutcomePosture};
use crate::runtime::tests::support::stateful_bridge_task_runtime;
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};

use super::{declare, inspection_basis, WorthQueryInspectionOutcome};

#[test]
fn operational_and_rich_inspection_preserve_the_same_receipt_and_operational_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("ordinary-inspection")
        .expect("workspace should open");
    let completion = declare_read(identity_read)
        .expect("read should declare")
        .using(current())
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("read should complete");

    let basis = scoped_inspection_basis("policy-equivalence");
    let operational = declare(&completion)
        .using(inspection_basis(basis.clone()))
        .run(&workspace);
    let rich = declare(&completion)
        .with_rich_inspection()
        .using(inspection_basis(basis))
        .run(&workspace);

    let operational = completion_or_panic(&operational);
    let rich = completion_or_panic(&rich);
    assert_eq!(operational.receipt(), rich.receipt());
    assert!(operational.materialization().is_none());
    assert!(rich.materialization().is_some());
    assert_eq!(operational.counters().materialization_attempt_count(), 0);
    assert_eq!(
        operational
            .estimated_cost()
            .bridge_envelope_assembly_count(),
        0
    );
    assert_eq!(rich.counters().materialization_attempt_count(), 1);
    assert_eq!(rich.estimated_cost().bridge_envelope_assembly_count(), 1);
    assert_eq!(rich.counters().materialization_completed_count(), 1);
}

#[test]
fn ordinary_inspection_implements_common_outcome_navigation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("ordinary-inspection-navigation")
        .expect("workspace should open");
    let completion = declare_read(identity_read)
        .expect("read should declare")
        .using(current())
        .run(request_execution, &mut workspace)
        .into_result()
        .expect("read should complete");
    let outcome = declare(&completion)
        .using(inspection_basis(scoped_inspection_basis("navigation")))
        .run(&workspace);

    assert_eq!(outcome.posture(), WorthQueryOutcomePosture::Completed);
    assert!(outcome.is_completed());
    assert!(!outcome.is_advisory());
}

fn identity_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_detail(
        "Task",
        QuerySchemaView::new(
            "ordinary-inspection-read",
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

fn scoped_inspection_basis(label: &str) -> crate::basis_lifecycle::ScopedInspectionBasis {
    basis_lifecycle()
        .historical_snapshot(format!("ordinary-inspection-{label}"), true)
        .inspect()
        .expect("inspection basis should admit")
}

fn completion_or_panic(
    outcome: &WorthQueryInspectionOutcome,
) -> &super::WorthQueryInspectionCompletion {
    if let Some(stop) = outcome.stop() {
        panic!(
            "inspection stopped at {:?}: {}",
            stop.source(),
            stop.evidence_for_reporting()
        );
    }
    if let Some(unavailable) = outcome.unavailable() {
        panic!(
            "inspection unavailable at {:?}: {}",
            unavailable.source(),
            unavailable.message()
        );
    }
    outcome.settled().expect("inspection should settle")
}
