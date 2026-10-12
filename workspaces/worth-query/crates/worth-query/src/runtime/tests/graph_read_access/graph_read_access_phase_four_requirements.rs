use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    TraversalSelector, WorthQueryGraphReadDomainOperationDeclaration, WorthQueryPredicateOperand,
};
use crate::runtime::{
    derive_graph_read_access_requirements, explain_boolean_selectivity_shape_for_family,
    explain_graph_read_access_requirements_for_family,
    explain_graph_read_access_requirements_for_family_with_operation_lookup,
    explain_graph_read_access_shape_for_family, QuerySchemaView, ScalarAspectType, SchemaFieldView,
    SchemaRelationView, WorthQueryGraphReadAccessComplexityContract,
    WorthQueryGraphReadAccessInvalidationBasis, WorthQueryGraphReadAccessMemoryEstimateBasis,
    WorthQueryGraphReadAccessRebuildBasis, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadFanoutPosture, WorthQueryGraphReadTraversalOperator,
};
use crate::runtime::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectKey, FieldKey};

use crate::runtime::tests::graph_read_access::support;

use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("test aspect key should be valid")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value).expect("test field key should be valid")
}

#[test]
fn equivalent_access_shapes_derive_stable_requirement_sets() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-four.parity")
        .expect("runtime should open workspace");
    let first = manager_traversal_family(&mut workspace, "phase-four-parity-first", "manager");
    let second = manager_traversal_family(&mut workspace, "phase-four-parity-second", "manager");

    let first_requirements = explain_graph_read_access_requirements_for_family(&first)
        .expect("first requirements should derive");
    let second_requirements = explain_graph_read_access_requirements_for_family(&second)
        .expect("second requirements should derive");

    assert_eq!(first_requirements.rows(), second_requirements.rows());
    assert_eq!(
        first_requirements.digest().render_support_hex(),
        second_requirements.digest().render_support_hex()
    );
    assert_eq!(
        first_requirements.diagnostic_canonical_parts(),
        second_requirements.diagnostic_canonical_parts()
    );
    assert!(first_requirements
        .requires_kind(WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency));
    assert!(first_requirements
        .requires_kind(WorthQueryGraphReadAccessRequirementKind::TraversalWorkset));
    let adjacency = first_requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency)
        .expect("directional adjacency row should exist");
    assert_eq!(adjacency.relation_name(), Some("manager"));
    assert_eq!(adjacency.relation_depth(), Some(2));
    assert_eq!(
        adjacency.fanout_posture(),
        Some(&WorthQueryGraphReadFanoutPosture::SingleRelation)
    );
    let authority = adjacency
        .relation_authority()
        .expect("relation authority should be derived from schema proof");
    assert_eq!(authority.relation_name(), "manager");
    assert_ne!(authority.schema_basis_digest().bytes(), &[0; 32]);
    assert_eq!(
        adjacency.invalidation_basis(),
        &WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta
    );
    assert_eq!(
        adjacency.complexity_contract(),
        &WorthQueryGraphReadAccessComplexityContract::DirectionalRelationLookup
    );
    assert_eq!(
        adjacency.memory_estimate_basis(),
        &WorthQueryGraphReadAccessMemoryEstimateBasis::RelationDegreeBound
    );
}

#[test]
fn changing_one_traversal_relation_localizes_requirement_row_change() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-four.localization")
        .expect("runtime should open workspace");
    let manager = manager_traversal_family(&mut workspace, "phase-four-manager", "manager");
    let mentor = manager_traversal_family(&mut workspace, "phase-four-mentor", "mentor");

    let manager_requirements = explain_graph_read_access_requirements_for_family(&manager)
        .expect("manager requirements should derive");
    let mentor_requirements = explain_graph_read_access_requirements_for_family(&mentor)
        .expect("mentor requirements should derive");
    let manager_rows = semantic_requirement_rows(manager_requirements.rows());
    let mentor_rows = semantic_requirement_rows(mentor_requirements.rows());

    assert_eq!(
        manager_rows.keys().collect::<Vec<_>>(),
        mentor_rows.keys().collect::<Vec<_>>()
    );

    let changed_slots = manager_rows
        .iter()
        .filter_map(|(slot, manager_digest)| {
            let mentor_digest = mentor_rows
                .get(slot)
                .expect("semantic slot should exist in mentor rows");
            (manager_digest != mentor_digest).then_some(slot.as_str())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changed_slots,
        [
            "slot:directional_adjacency:forward:2:single_relation:none:none:declaration_traversal:none:none:none",
            "slot:traversal_workset:forward:2:single_relation:none:none:declaration_traversal:none:none:none",
            "slot:visited_set:forward:2:single_relation:none:none:declaration_traversal:none:none:none"
        ]
    );
}

fn semantic_requirement_rows(
    rows: &[crate::runtime::WorthQueryGraphReadAccessRequirementRow],
) -> BTreeMap<String, String> {
    rows.iter()
        .map(|row| (row.semantic_slot_key(), row.digest_part()))
        .collect()
}

#[path = "graph_read_access_phase_four_requirements/predicate_and_ordering_shapes_add_typed_support_rows.rs"]
mod predicate_and_ordering_shapes_add_typed_support_rows;

#[test]
fn requirement_derivation_consumes_access_and_selectivity_proofs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-four.proof-chain")
        .expect("runtime should open workspace");
    let family = manager_traversal_family(&mut workspace, "phase-four-proof-chain", "manager");

    let explanation =
        explain_graph_read_access_shape_for_family(&family).expect("access shape should derive");
    let selectivity =
        explain_boolean_selectivity_shape_for_family(&family).expect("selectivity should derive");
    let requirements =
        derive_graph_read_access_requirements(explanation.access_shape(), &selectivity);

    assert_eq!(
        requirements.read_graph_digest().render_hex(),
        explanation
            .access_shape()
            .operation_resolution()
            .read_graph_canonical_digest()
            .render_hex()
    );
    assert_eq!(
        requirements.access_shape_digest().render_hex(),
        explanation.access_shape().digest().as_str()
    );
    assert_eq!(
        requirements.selectivity_shape_digest().render_hex(),
        selectivity.digest().as_str()
    );
}

#[test]
fn registry_backed_domain_operations_derive_requirements_through_explicit_registry() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("graph-read-access.phase-four.registry-requirements")
        .expect("runtime should open workspace");
    let operation = visible_face_neighborhood_operation();
    let family = domain_operation_family(
        &mut workspace,
        "phase-four-registry-requirements",
        operation.clone(),
    );
    let registry = WorthQueryGraphReadOperationRegistry::admit([
        WorthQueryGraphReadOperationRegistration::for_declared_operation(&operation)
            .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::SuccessorWalk),
    ])
    .expect("domain operation registration should admit");

    let requirements =
        explain_graph_read_access_requirements_for_family_with_operation_lookup(&family, &registry)
            .expect("registered requirements should derive");

    assert!(
        requirements.requires_kind(WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency)
    );
    assert!(requirements.requires_kind(WorthQueryGraphReadAccessRequirementKind::TraversalWorkset));
    assert!(requirements
        .rows()
        .iter()
        .any(|row| row.relation_name() == Some("manager")));
}

fn manager_traversal_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    family_name: &str,
    relation: &str,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.anchored_collection(
                "user",
                two_relation_schema(),
                |query| {
                    query
                        .traverse(traversal(relation, 2))
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("manager traversal family should admit")
}

fn domain_operation_family(
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
                        .project(field("identity", "id"))
                },
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("domain operation family should admit")
}

fn visible_face_neighborhood_operation() -> WorthQueryGraphReadDomainOperationDeclaration {
    WorthQueryGraphReadDomainOperationDeclaration::new(
        "worth.geometry.visible_face_neighborhood",
        1,
        "worth.geometry",
    )
    .expect("operation key should admit")
    .admit_relation_reference("manager")
    .expect("operation reference should admit")
    .requires_support_family("worth.geometry.visible_face_neighborhood.access")
    .expect("support family should admit")
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

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn two_relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-four-two-relation",
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
            SchemaFieldView::new(
                crate::facade::foundation::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::facade::foundation::FieldName::new("value")
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
