use super::support::graph_index_inventory::runtime_profiles::{
    profile_with_ephemeral_graph_support, workspace_with_graph_support,
};
use super::support::graph_read_access_cost_model::simple_traversal_family;
use crate::runtime::{
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessRequirementKind,
};

#[test]
fn ephemeral_plan_keeps_exact_native_requirement_rows_without_allocation_forecasts() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut workspace = workspace_with_graph_support(
        execution,
        "ephemeral-requirements",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = simple_traversal_family(&mut workspace, "ephemeral-requirements");
    let review = workspace.read_family_intent(&family).review().unwrap();
    let plan = review.graph_read_access_plan().unwrap();
    let ephemeral = plan.ephemeral_index_plan().unwrap();
    assert_eq!(
        plan.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex
    );
    let expected: Vec<_> = plan
        .graph_index_support()
        .matches()
        .iter()
        .filter(|row| {
            row.resolved_admission_posture()
                == &WorthQueryGraphReadAccessAdmissionPosture::BoundedEphemeralIndex
        })
        .collect();
    assert_eq!(ephemeral.required_allocations().len(), expected.len());
    assert!(!expected.is_empty());
    for (row, expected) in ephemeral.required_allocations().iter().zip(expected) {
        assert_eq!(row.requirement_kind(), expected.requirement_kind());
        assert_eq!(
            row.requirement_row_digest(),
            expected.requirement_row_digest()
        );
    }
}
