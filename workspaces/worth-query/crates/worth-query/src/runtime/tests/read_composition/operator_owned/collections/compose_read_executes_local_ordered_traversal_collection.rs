//! compose read executes local ordered traversal collection.
use super::*;

#[test]
fn compose_read_executes_local_ordered_traversal_collection() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = read_runtime(request_execution)
        .workspace("runtime.read-composition.collection-local")
        .expect("read-backed runtime should open a workspace");

    let result = workspace
        .compose_read(request_execution, |read| {
            read.local_collection(
                "user",
                manager_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("profile", "display_name")
                                .expect("name projection should build"),
                        )
                        .traverse(
                            TraversalSelector::bounded("manager", 1)
                                .expect("bounded traversal should build"),
                        )
                        .order_by(
                            OrderingSelector::ascending("profile", "display_name")
                                .expect("ordering should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new(
                                "profile",
                                "display_name",
                                "display_name",
                            )
                            .expect("name result-shape field should build"),
                        )
                },
            )
        })
        .expect("ordered local collection read should execute");

    assert!(!result.rows().is_empty());
    assert_collection_receipt(&result, WorthQueryReadScopeClass::LocalNeighborhood);
    assert!(result.receipt().built_in_operator_coverage().is_empty());
    assert_descriptor_admitted_synthetic_runtime_relationship_proof(
        &result,
        2,
        &direct_edge_synthetic_runtime_surfaces(),
    );
}
