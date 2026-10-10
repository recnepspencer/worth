use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, RelationName, TraversalSelector,
    WorthQueryGraphReadDomainOperationDeclaration,
};
use crate::runtime::{
    resolve_graph_read_operations_for_family_with_operation_lookup, QuerySchemaView,
    ScalarAspectType, SchemaFieldView, SchemaRelationView,
    WorthQueryGraphReadResolvedOperationFamily, WorthQueryGraphReadResolvedOperationKind,
    WorthQueryGraphReadTraversalOperator, WorthQueryReadBuiltInOperator,
};
use crate::runtime::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
    WorthQueryGraphReadRegistryAdmissionError,
};

use crate::runtime::tests::graph_read_access::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn built_in_operation_resolution_is_stable_across_equivalent_read_declarations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-three.built-in-parity")
        .expect("runtime should open workspace");
    let first = direct_edge_family(&mut workspace, "direct-edge-first");
    let second = direct_edge_family(&mut workspace, "direct-edge-second");
    let registry = WorthQueryGraphReadOperationRegistry::empty();

    let first_resolution =
        resolve_graph_read_operations_for_family_with_operation_lookup(&first, &registry)
            .expect("first direct edge should resolve")
            .resolved()
            .expect("first direct edge should be resolved")
            .clone();
    let second_resolution =
        resolve_graph_read_operations_for_family_with_operation_lookup(&second, &registry)
            .expect("second direct edge should resolve")
            .resolved()
            .expect("second direct edge should be resolved")
            .clone();

    assert_eq!(
        first_resolution.operations(),
        second_resolution.operations()
    );
    assert_eq!(
        first_resolution.operations()[0].family(),
        &WorthQueryGraphReadResolvedOperationFamily::BuiltIn
    );
    assert_eq!(
        first_resolution.operations()[0].kind(),
        &WorthQueryGraphReadResolvedOperationKind::BuiltIn(
            WorthQueryReadBuiltInOperator::DirectEdge
        )
    );
}

#[test]
fn declared_domain_operation_matches_registered_reference_order_once() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-three.order-equivalence")
        .expect("runtime should open workspace");
    let operation = dual_chain_operation();
    let family =
        declared_two_relation_domain_family(&mut workspace, "order-equivalence", operation.clone());
    let registry = WorthQueryGraphReadOperationRegistry::admit([
        WorthQueryGraphReadOperationRegistration::for_declared_operation(&operation)
            .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::SuccessorWalk),
    ])
    .expect("registry should admit declared domain operation");

    let outcome =
        resolve_graph_read_operations_for_family_with_operation_lookup(&family, &registry)
            .expect("registered operation should resolve");
    let resolution = outcome.resolved().expect("outcome should be resolved");

    assert_eq!(resolution.operations().len(), 1);
    assert_eq!(
        resolution.operations()[0].family(),
        &WorthQueryGraphReadResolvedOperationFamily::DomainRegistered
    );
}

#[test]
fn duplicate_registry_operation_keys_are_denied_at_admission() {
    let operation = dual_chain_operation();
    let registration = WorthQueryGraphReadOperationRegistration::for_declared_operation(&operation)
        .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::SuccessorWalk);

    let error = WorthQueryGraphReadOperationRegistry::admit([registration.clone(), registration])
        .expect_err("duplicate operation key should deny registry admission");

    assert_eq!(
        error,
        WorthQueryGraphReadRegistryAdmissionError::DuplicateOperationKey
    );
}

#[test]
fn ambiguous_registry_reference_admission_is_denied() {
    let first = dual_chain_operation();
    let second = WorthQueryGraphReadDomainOperationDeclaration::new(
        "worth.geometry.different_dual_chain",
        1,
        "worth.geometry",
    )
    .expect("operation key should admit")
    .admit_relation_reference("manager")
    .expect("reference should admit")
    .admit_relation_reference("mentor")
    .expect("reference should admit")
    .requires_support_family("worth.geometry.different_dual_chain.access")
    .expect("support should admit");

    let error = WorthQueryGraphReadOperationRegistry::admit([
        WorthQueryGraphReadOperationRegistration::for_declared_operation(&first)
            .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::SuccessorWalk),
        WorthQueryGraphReadOperationRegistration::for_declared_operation(&second)
            .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::AnchoredFrontier),
    ])
    .expect_err("same admitted refs with different operation keys should deny");

    assert_eq!(
        error,
        WorthQueryGraphReadRegistryAdmissionError::AmbiguousDomainReferenceAdmission
    );
}

fn direct_edge_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    family_name: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_direct_edge_collection(
                "user",
                two_relation_schema(),
                relation("manager"),
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("direct-edge family should be admitted")
}

fn declared_two_relation_domain_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    family_name: &str,
    operation: WorthQueryGraphReadDomainOperationDeclaration,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.anchored_collection(
                "user",
                two_relation_schema(),
                |query| {
                    query
                        .domain_graph_operation(operation)
                        .traverse(traversal("manager", 2))
                        .traverse(traversal("mentor", 2))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("two-relation domain family should be admitted")
}

fn dual_chain_operation() -> WorthQueryGraphReadDomainOperationDeclaration {
    WorthQueryGraphReadDomainOperationDeclaration::new(
        "worth.geometry.dual_chain",
        1,
        "worth.geometry",
    )
    .expect("operation key should admit")
    .admit_relation_reference("mentor")
    .expect("reference should admit")
    .admit_relation_reference("manager")
    .expect("reference should admit")
    .requires_support_family("worth.geometry.dual_chain.access")
    .expect("support should admit")
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).expect("field selector should build")
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered)
        .expect("result-shape field should build")
}

fn traversal(name: &str, depth: u8) -> TraversalSelector {
    TraversalSelector::bounded(name, depth).expect("traversal selector should build")
}

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn two_relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-three-two-relation",
        [
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("id")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("profile")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("display_name")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [
            SchemaRelationView::new(
                crate::facade::foundation::RelationName::new("manager")
                    .expect("schema relation literal must be valid"),
                2,
            ),
            SchemaRelationView::new(
                crate::facade::foundation::RelationName::new("mentor")
                    .expect("schema relation literal must be valid"),
                2,
            ),
        ],
    )
}
