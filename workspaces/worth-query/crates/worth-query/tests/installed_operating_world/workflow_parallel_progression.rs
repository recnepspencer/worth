use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    divergent_frontier_workspace, missing_parallel_provider_workspace,
    nondeterministic_workflow_workspace, serial_parallel_provider_workspace, workflow_workspace,
    GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn admitted_parallel_frontier_retains_lower_proof_and_converges_with_serial_trace() {
    let serial = complete_serial_trace("workflow-serial-reference");
    let mut workspace = workflow_workspace("workflow-parallel-frontier").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance_admitted_frontier(
            [
                (
                    "right".into(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                ),
                (
                    "left".into(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                ),
            ],
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap();
    let trace = run.complete().unwrap();

    assert_eq!(trace.semantic_identity(), serial);
    assert_eq!(trace.counters().parallel_admission_checks, 1);
    for stage in trace
        .stage_receipts()
        .iter()
        .filter(|receipt| matches!(receipt.stage_identity(), "left" | "right"))
    {
        let proof = stage.parallel_admission().unwrap();
        assert_eq!(proof.run_identity(), stage.run_identity());
        assert_eq!(
            proof
                .frontier()
                .iter()
                .map(|stage| stage.stage_identity())
                .collect::<Vec<_>>(),
            ["left", "right"]
        );
        assert!(proof.lower_receipt().is_parallel_admitted());
        assert_eq!(stage.counters().parallel_admission_checks, 0);
    }
}

#[test]
fn nondeterministic_lowering_cannot_enter_parallel_progression() {
    let mut workspace = nondeterministic_workflow_workspace("workflow-nondeterministic").unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap();
    let denial = match run.advance_admitted_frontier(
        [
            (
                "left".into(),
                domain::WorthQueryWorkflowValue::Text("start".into()),
            ),
            (
                "right".into(),
                domain::WorthQueryWorkflowValue::Text("start".into()),
            ),
        ],
        &mut workspace,
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("nondeterministic lowering did not produce an exact denial"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::NonDeterministicLowering
    );
    assert_eq!(denial.counters().parallel_admission_checks, 0);
}

#[test]
fn parallel_workflow_requires_an_exact_admission_provider_at_runtime_construction() {
    let denial = match missing_parallel_provider_workspace("workflow-parallel-provider-missing") {
        Ok(_) => panic!("parallel workflow must not install without its lower admission provider"),
        Err(denial) => denial,
    };
    assert!(denial
        .message()
        .contains("parallel workflow operation and parallel-admission provider sets differ"));
}

#[test]
fn lower_runtime_parallel_denial_stops_before_frontier_graph_or_executor_work() {
    let mut workspace =
        serial_parallel_provider_workspace("workflow-parallel-lower-denial").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap();
    let denial = match run.advance_admitted_frontier(
        [
            (
                "left".into(),
                domain::WorthQueryWorkflowValue::Text("start".into()),
            ),
            (
                "right".into(),
                domain::WorthQueryWorkflowValue::Text("start".into()),
            ),
        ],
        &mut workspace,
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("serial lower-runtime receipt did not produce an exact denial"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::ParallelNotAdmitted(
            worth_signal::facade::adapters::FrontierRouteSerialFallbackReason::BelowMinStageWidth,
        )
    );
    assert_eq!(denial.counters().parallel_admission_checks, 1);
    assert_eq!(denial.counters().stage_executor_contacts, 1);
    assert_eq!(denial.counters().graph_read_contacts, 0);
    assert_eq!(denial.counters().touch_effect_contacts, 0);
}

#[test]
fn parallel_frontier_accepts_ready_incomparable_stages_with_distinct_predecessors() {
    let mut workspace = divergent_frontier_workspace("workflow-divergent-frontier").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "bridge",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance_admitted_frontier(
            [
                (
                    "left".into(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                ),
                (
                    "right".into(),
                    domain::WorthQueryWorkflowValue::Text("bridge".into()),
                ),
            ],
            &mut workspace,
        )
        .unwrap();
    let proof = run
        .receipts()
        .iter()
        .find(|receipt| receipt.stage_identity() == "left")
        .unwrap()
        .parallel_admission()
        .unwrap();
    assert_ne!(
        proof.frontier()[0].predecessor_receipt_identities(),
        proof.frontier()[1].predecessor_receipt_identities()
    );
}

fn complete_serial_trace(name: &str) -> String {
    let mut workspace = workflow_workspace(name).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap()
        .semantic_identity()
        .to_string()
}
