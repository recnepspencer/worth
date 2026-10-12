//! predicate and ordering shapes add typed support rows.
use super::*;

#[test]
fn predicate_and_ordering_shapes_add_typed_support_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-four.predicate-ordering")
        .expect("runtime should open workspace");
    let family = workspace
        .define_read_family("phase-four-predicate-ordering", |read| {
            read.explicit_broad_search_collection(
                "user",
                two_relation_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 2))
                        .where_equal(equality("status", "value", "active"))
                        .project(field("identity", "id"))
                        .order_by(
                            OrderingSelector::ascending("profile", "display_name")
                                .expect("ordering should build"),
                        )
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("predicate ordering family should admit");

    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");

    assert!(requirements.requires_kind(WorthQueryGraphReadAccessRequirementKind::PredicateSupport));
    assert!(requirements.requires_kind(WorthQueryGraphReadAccessRequirementKind::OrderingSupport));
    let predicate_row = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::PredicateSupport)
        .expect("predicate support row should exist");
    let ordering_row = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::OrderingSupport)
        .expect("ordering support row should exist");
    assert_eq!(
        predicate_row.rebuild_basis(),
        &WorthQueryGraphReadAccessRebuildBasis::SelectivityProof
    );
    assert_eq!(predicate_row.predicate_field_authorities().len(), 1);
    assert_eq!(
        predicate_row.predicate_field_authorities()[0].native_aspect_key(),
        &aspect_key("status")
    );
    assert_eq!(
        predicate_row.predicate_field_authorities()[0].native_field_key(),
        &field_key("value")
    );
    assert_eq!(ordering_row.ordering_field_authorities().len(), 1);
    assert_eq!(
        ordering_row.ordering_field_authorities()[0].native_aspect_key(),
        &aspect_key("profile")
    );
    assert_eq!(
        ordering_row.ordering_field_authorities()[0].native_field_key(),
        &field_key("display_name")
    );
}
