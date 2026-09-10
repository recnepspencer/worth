use worth_query::facade::domain;

use super::installed_operation_fixture::{
    workflow_graph_projection_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn workflow_executor_consumes_execution_bound_separate_graph_projection() {
    let mut workspace = workflow_graph_projection_workspace("workflow-graph-projection").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();
    let run = bound
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
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
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
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap();
    let trace = run.complete().unwrap();
    let publish = trace.stage_receipts().last().unwrap();
    let remote_receipt = publish
        .graph_receipts()
        .iter()
        .find(|receipt| receipt.role() == "remote-a")
        .unwrap();

    assert_eq!(
        remote_receipt
            .graph_read_product()
            .unwrap()
            .rows()
            .next()
            .unwrap()
            .entity_identity(),
        "workflow-remote-row"
    );
    assert_eq!(
        publish.warnings(),
        [domain::WorthQueryWorkflowStageWarning::Advisory(
            "remote-a-rows=1;first=workflow-remote-row".into()
        )]
    );
    assert_eq!(trace.counters().graph_read_contacts, 2);
}
