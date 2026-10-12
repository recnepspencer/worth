use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, RelationName, TraversalSelector,
};
use crate::runtime::{
    ScalarAspectType, WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadPolicyTenantPosture,
    WorthQueryGraphReadRelationshipProofBindingPosture, WorthQueryGraphReadResolvedOperationFamily,
    WorthQueryGraphReadResolvedOperationKind, WorthQueryGraphReadRootPosture,
    WorthQueryGraphReadTraversalOperator, WorthQueryReadScopeClass,
};

use crate::runtime::tests::graph_read_access::support;
mod phase_one_schema_fixtures;

use phase_one_schema_fixtures::*;
use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn graph_read_access_shape_explains_anchored_frontier_without_executing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.frontier")
        .expect("runtime should open workspace");
    let family = workspace
        .define_read_family("tenant-face-neighborhood", |read| {
            read.anchored_frontier_collection(
                "user",
                frontier_schema(),
                [relation("manager"), relation("mentor")],
                2,
                |query| {
                    query
                        .project(field("identity", "id"))
                        .project(field("profile", "display_name"))
                },
                |shape| {
                    shape
                        .field(result_field("identity", "id", "id"))
                        .field(result_field("profile", "display_name", "display_name"))
                },
            )
        })
        .expect("frontier family should be admitted");

    let explanation = workspace
        .explain_graph_read_access_shape(&family)
        .expect("frontier access shape should explain");
    let shape = explanation.access_shape();

    assert_eq!(
        explanation.read_family_digest(),
        family.family_digest(),
        "explanation should bind the reusable family identity"
    );
    assert_eq!(
        shape.root_posture(),
        &WorthQueryGraphReadRootPosture::Anchored
    );
    assert_eq!(
        shape.scope_class(),
        &WorthQueryReadScopeClass::AnchoredExpansion
    );
    assert_eq!(
        shape.traversal_operators(),
        [WorthQueryGraphReadTraversalOperator::AnchoredFrontier]
    );
    assert_eq!(shape.max_depth(), 2);
    assert_eq!(
        shape.fanout_posture(),
        &WorthQueryGraphReadFanoutPosture::Frontier
    );
    assert_eq!(
        shape.relationship_proof_posture(),
        &WorthQueryGraphReadRelationshipProofBindingPosture::DescriptorAdmittedSyntheticRuntime
    );
    assert_eq!(
        shape
            .operation_resolution()
            .policy_tenant_proof_binding()
            .policy_tenant_posture(),
        &WorthQueryGraphReadPolicyTenantPosture::SyntheticRuntimeCurrentRead
    );
    assert_eq!(
        explanation.admitted_schema_references().relations().len(),
        2
    );
    assert!(
        explanation
            .admitted_schema_references()
            .projections()
            .iter()
            .all(|field| field.kind().native_family() == ScalarAspectType::String),
        "happy-path result fields should be proven present in the frozen schema"
    );
    assert!(explanation.derivation_counters().is_derivation_only());
    assert_eq!(
        explanation
            .derivation_counters()
            .schema_reference_rows_admitted(),
        5,
        "two relations, two projections, and the canonical identity ordering must be admitted"
    );
    assert!(explanation
        .explain()
        .contains("operators=anchored_frontier"));
}

#[test]
fn graph_read_access_shape_resolves_declared_traversal_before_shape_derivation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.declared-traversal")
        .expect("runtime should open workspace");
    let family = workspace
        .define_read_family("declared-manager-path", |read| {
            read.anchored_collection(
                "user",
                manager_schema(),
                |query| {
                    query
                        .traverse(traversal("manager", 2))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("declared traversal family should be admitted");

    let explanation = workspace
        .explain_graph_read_access_shape(&family)
        .expect("declared traversal access shape should explain");
    let shape = explanation.access_shape();
    let operations = shape.operation_resolution().operations();

    assert_eq!(
        operations.len(),
        1,
        "plain declaration traversal should be resolved exactly once"
    );
    assert_eq!(
        operations[0].family(),
        &WorthQueryGraphReadResolvedOperationFamily::DeclaredTraversal
    );
    assert_eq!(
        operations[0].kind(),
        &WorthQueryGraphReadResolvedOperationKind::DeclarationTraversal
    );
    assert_eq!(
        shape.traversal_operators(),
        [WorthQueryGraphReadTraversalOperator::DeclarationTraversal]
    );
}

#[test]
fn graph_read_access_shape_keeps_direct_edge_distinct_from_successor_walk() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.locality")
        .expect("runtime should open workspace");
    let direct = workspace
        .define_read_family("manager-direct", |read| {
            read.local_direct_edge_collection(
                "user",
                manager_schema(),
                relation("manager"),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("direct edge should be admitted");
    let successor = workspace
        .define_read_family("manager-successor", |read| {
            read.local_successor_walk_collection(
                "user",
                manager_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("successor walk should be admitted");

    let direct_shape = workspace
        .explain_graph_read_access_shape(&direct)
        .expect("direct access shape should explain");
    let successor_shape = workspace
        .explain_graph_read_access_shape(&successor)
        .expect("successor access shape should explain");

    assert_ne!(
        direct_shape.access_shape().digest(),
        successor_shape.access_shape().digest(),
        "one-hop direct edge and bounded successor walk must not collapse"
    );
    assert_eq!(
        direct_shape.access_shape().traversal_operators(),
        [WorthQueryGraphReadTraversalOperator::DirectEdge]
    );
    assert_eq!(
        successor_shape.access_shape().traversal_operators(),
        [WorthQueryGraphReadTraversalOperator::SuccessorWalk]
    );
}

#[test]
fn graph_read_access_shape_digest_is_stable_across_frontier_relation_ordering() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.equivalence")
        .expect("runtime should open workspace");
    let manager_then_mentor = workspace
        .define_read_family("manager-mentor", |read| {
            read.anchored_frontier_collection(
                "user",
                frontier_schema(),
                [relation("manager"), relation("mentor")],
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("frontier family should be admitted");
    let mentor_then_manager = workspace
        .define_read_family("mentor-manager", |read| {
            read.anchored_frontier_collection(
                "user",
                frontier_schema(),
                [relation("mentor"), relation("manager")],
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("frontier family should be admitted");

    let first_shape = workspace
        .explain_graph_read_access_shape(&manager_then_mentor)
        .expect("first frontier access shape should explain");
    let second_shape = workspace
        .explain_graph_read_access_shape(&mentor_then_manager)
        .expect("second frontier access shape should explain");

    assert_eq!(
        first_shape.access_shape().digest(),
        second_shape.access_shape().digest(),
        "semantically equivalent frontier relation ordering should not change access shape"
    );
    let relation_rows = first_shape
        .admitted_schema_references()
        .relations()
        .iter()
        .map(|relation| {
            (
                relation.relation_name().as_str().to_string(),
                relation.direction().as_str(),
                relation.depth(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        relation_rows,
        vec![
            ("manager".to_string(), "forward", 2),
            ("mentor".to_string(), "forward", 2)
        ]
    );
}

#[test]
fn graph_read_access_shape_digest_changes_with_result_shape_breadth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-one.result-pressure")
        .expect("runtime should open workspace");
    let narrow = workspace
        .define_read_family("narrow-result", |read| {
            read.local_direct_edge_collection(
                "user",
                wide_schema(),
                relation("manager"),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("narrow family should be admitted");
    let wide = workspace
        .define_read_family("wide-result", |read| {
            read.local_direct_edge_collection(
                "user",
                wide_schema(),
                relation("manager"),
                |query| {
                    query
                        .project(field("identity", "id"))
                        .project(field("profile", "display_name"))
                        .project(field("profile", "title"))
                        .project(field("profile", "department"))
                },
                |shape| {
                    shape
                        .field(result_field("identity", "id", "id"))
                        .field(result_field("profile", "display_name", "display_name"))
                        .field(result_field("profile", "title", "title"))
                        .field(result_field("profile", "department", "department"))
                },
            )
        })
        .expect("wide family should be admitted");

    let narrow_shape = workspace
        .explain_graph_read_access_shape(&narrow)
        .expect("narrow access shape should explain");
    let wide_shape = workspace
        .explain_graph_read_access_shape(&wide)
        .expect("wide access shape should explain");

    assert_ne!(
        narrow_shape.access_shape().digest(),
        wide_shape.access_shape().digest(),
        "result-shape breadth is access pressure and must affect the shape"
    );
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn traversal(name: &str, depth: u8) -> TraversalSelector {
    TraversalSelector::bounded(name, depth).expect("traversal selector should build")
}
