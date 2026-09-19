use super::installed_operation_fixture::{
    invalid_workflow_workspace, mismatched_workflow_determinism_workspace,
    mismatched_workflow_lowering_workspace, reversed_workflow_workspace, workflow_workspace,
    GeometryDomain, InvalidWorkflow, ReadFamily, WorkflowRead,
};

#[test]
fn workflow_executor_lowering_must_match_installed_operation_semantics() {
    let denial = match mismatched_workflow_lowering_workspace("workflow-lowering-mismatch") {
        Ok(_) => panic!("foreign workflow lowering family must not install"),
        Err(denial) => denial,
    };
    assert!(denial
        .message()
        .contains("workflow stage executor lowering family disagrees with installed semantics"));
}

#[test]
fn workflow_executor_determinism_must_match_installed_operation_semantics() {
    let denial = match mismatched_workflow_determinism_workspace("workflow-determinism-mismatch") {
        Ok(_) => panic!("workflow executor determinism drift must not install"),
        Err(denial) => denial,
    };
    assert!(denial
        .message()
        .contains("workflow executor determinism disagrees with installed semantics"));
}

#[test]
fn malformed_workflow_graphs_fail_before_runtime_construction() {
    for (name, invalid, expected) in [
        (
            "cycle",
            InvalidWorkflow::Cycle,
            "cyclic-or-unreachable-workflow-stage",
        ),
        (
            "missing",
            InvalidWorkflow::MissingPredecessor,
            "missing-workflow-predecessor",
        ),
        (
            "duplicate",
            InvalidWorkflow::DuplicateStage,
            "duplicate-workflow-stage-identity",
        ),
        (
            "extra-root",
            InvalidWorkflow::ExtraRoot,
            "workflow-non-entry-root",
        ),
        (
            "dead-end",
            InvalidWorkflow::IncompleteTerminalPath,
            "incomplete-workflow-terminal-path",
        ),
        (
            "undeclared-domain",
            InvalidWorkflow::UndeclaredRequiredDomain,
            "workflow-stage-references-undeclared-required-domain",
        ),
        (
            "unused-operation-graph-read",
            InvalidWorkflow::UnusedOperationGraphRead,
            "workflow-graph-read-closure-mismatch",
        ),
    ] {
        let denial = match invalid_workflow_workspace(name, invalid) {
            Ok(_) => panic!("malformed workflow must deny before runtime construction"),
            Err(denial) => denial,
        };
        assert!(
            denial.message().contains(expected),
            "{invalid:?} denied for the wrong reason: {}",
            denial.message()
        );
    }
}

#[test]
fn installed_workflow_graph_converges_across_stage_declaration_order() {
    let direct = installed_graph("workflow-graph-direct", false);
    let reversed = installed_graph("workflow-graph-reversed", true);
    assert_eq!(direct, reversed);
    assert_eq!(direct, ["left", "publish", "right", "start"]);
}

fn installed_graph(name: &str, reversed: bool) -> Vec<String> {
    let mut workspace = if reversed {
        reversed_workflow_workspace(name)
    } else {
        workflow_workspace(name)
    }
    .unwrap();
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
        .installed_graph()
        .stages()
        .iter()
        .map(|stage| stage.identity().to_string())
        .collect()
}
