use crate::facade::foundation::{
    AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, OrderingSelector,
    RelationName, TraversalSelector, WorthQueryGraphReadDomainOperationDeclaration,
    WorthQueryPredicateOperand,
};
use crate::runtime::{
    derive_graph_read_access_requirements, explain_boolean_selectivity_shape_for_family,
    explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup,
    explain_graph_read_access_requirements_for_family, explain_graph_read_access_shape_for_family,
    try_derive_graph_read_access_requirements, QuerySchemaView, ScalarAspectType, SchemaFieldView,
    SchemaRelationView, WorthQueryGraphReadAccessRequirementDerivationError,
    WorthQueryGraphReadAccessRequirementExplanationOutcome,
    WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadOperationUnsupportedShapeDeclaration, WorthQueryGraphReadTraversalOperator,
};
use crate::runtime::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
};
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
fn checked_requirement_derivation_rejects_mismatched_proof_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-four.checked-derivation",
    );
    let manager = traversal_family(&mut workspace, "phase-four-manager-proof", "manager");
    let mentor = traversal_family(&mut workspace, "phase-four-mentor-proof", "mentor");

    let manager_access =
        explain_graph_read_access_shape_for_family(&manager).expect("manager access should derive");
    let mentor_selectivity = explain_boolean_selectivity_shape_for_family(&mentor)
        .expect("mentor selectivity should derive");
    let error = try_derive_graph_read_access_requirements(
        manager_access.access_shape(),
        &mentor_selectivity,
    )
    .expect_err("checked derivation must reject unrelated proof artifacts");

    assert!(matches!(
        error,
        WorthQueryGraphReadAccessRequirementDerivationError::ReadGraphDigestMismatch { .. }
    ));
    assert_eq!(error.as_str(), "read_graph_digest_mismatch");
}

#[test]
fn trusted_requirement_derivation_preserves_canonical_parts_and_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-four.canonical-parts",
    );
    let family = traversal_family(&mut workspace, "phase-four-canonical-parts", "manager");

    let access =
        explain_graph_read_access_shape_for_family(&family).expect("access shape should derive");
    let selectivity =
        explain_boolean_selectivity_shape_for_family(&family).expect("selectivity should derive");
    let requirements = derive_graph_read_access_requirements(access.access_shape(), &selectivity);

    assert_eq!(
        requirements.diagnostic_canonical_parts(),
        requirements.diagnostic_canonical_parts()
    );
    assert_eq!(
        requirements.digest().render_support_hex(),
        derive_graph_read_access_requirements(access.access_shape(), &selectivity)
            .digest()
            .render_support_hex()
    );
}

#[test]
fn predicate_and_ordering_authorities_name_exact_schema_fields() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-four.authorities",
    );
    let family = workspace
        .define_read_family("phase-four-authorities", |read| {
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
    let predicate = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::PredicateSupport)
        .expect("predicate support row should exist")
        .predicate_field_authorities();
    let ordering = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::OrderingSupport)
        .expect("ordering support row should exist")
        .ordering_field_authorities();

    assert_eq!(predicate.len(), 1);
    assert_eq!(predicate[0].native_aspect_key(), &aspect_key("status"));
    assert_eq!(predicate[0].native_field_key(), &field_key("value"));
    assert_eq!(predicate[0].field_kind(), "string");
    assert_ne!(predicate[0].schema_basis_digest().bytes(), &[0; 32]);
    assert_eq!(ordering.len(), 1);
    assert_eq!(ordering[0].native_aspect_key(), &aspect_key("profile"));
    assert_eq!(ordering[0].native_field_key(), &field_key("display_name"));
    assert_eq!(ordering[0].direction(), "ascending");
    assert_eq!(ordering[0].field_kind(), "string");
    assert_ne!(ordering[0].schema_basis_digest().bytes(), &[0; 32]);
}

#[test]
fn registry_requirement_outcome_exposes_registered_required_and_denied_paths() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-four.registry-outcomes",
    );
    let operation = visible_face_neighborhood_operation();
    let family = domain_operation_family(&mut workspace, operation.clone());
    let registered = WorthQueryGraphReadOperationRegistry::admit([
        WorthQueryGraphReadOperationRegistration::for_declared_operation(&operation)
            .lowers_to_traversal_operator(WorthQueryGraphReadTraversalOperator::SuccessorWalk),
    ])
    .expect("domain operation registration should admit");
    let denied = WorthQueryGraphReadOperationRegistry::empty()
        .with_unsupported_shape_for_operation(
            operation.key().clone(),
            WorthQueryGraphReadOperationUnsupportedShapeDeclaration::unsupported_shape(
                "phase-four-denied-domain-operation",
                "test registry rejects this domain operation",
            ),
        );

    let missing = explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup(
        &family,
        &WorthQueryGraphReadOperationRegistry::empty(),
    )
    .expect("missing capability should explain");
    let resolved = explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup(
        &family,
        &registered,
    )
    .expect("registered capability should explain");
    let unsupported =
        explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup(
            &family, &denied,
        )
        .expect("unsupported capability should explain");

    assert_eq!(missing.as_str(), "requires_access_capability_registration");
    assert!(matches!(
        missing,
        WorthQueryGraphReadAccessRequirementExplanationOutcome::RequiresAccessCapabilityRegistration(_)
    ));
    assert!(resolved
        .requirement_set()
        .expect("registered outcome should carry requirements")
        .requires_kind(WorthQueryGraphReadAccessRequirementKind::TraversalWorkset));
    assert!(matches!(
        unsupported,
        WorthQueryGraphReadAccessRequirementExplanationOutcome::DeniedUnsupportedShape(_)
    ));
}

#[test]
fn successor_walk_counters_are_exact_not_elapsed_time_proxies() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-four.exact-counters",
    );
    let family = workspace
        .define_read_family("phase-four-successor-walk-counters", |read| {
            read.local_successor_walk_collection(
                "user",
                relation_schema(),
                relation("manager"),
                2,
                |query| query.project(field("identity", "id")),
                |shape| shape.field(result_field("identity", "id", "id")),
            )
        })
        .expect("successor walk family should admit");

    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");
    let counters = requirements.counters();

    assert_eq!(counters.row_count(), requirements.rows().len());
    assert_eq!(counters.directional_adjacency_count(), 1);
    assert_eq!(counters.reverse_adjacency_count(), 0);
    assert_eq!(counters.traversal_workset_count(), 1);
    assert_eq!(counters.visited_set_count(), 1);
    assert_eq!(counters.dedup_set_count(), 0);
    assert_eq!(counters.workset_count(), 2);
    assert_eq!(counters.buffer_count(), 1);
    assert_eq!(counters.materialization_lifecycle_count(), 1);
}

fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> crate::runtime::WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace(name)
        .expect("runtime should open workspace")
}

fn traversal_family(
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
        .expect("traversal family should admit")
}

fn domain_operation_family(
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    operation: WorthQueryGraphReadDomainOperationDeclaration,
) -> crate::runtime::WorthQueryReadFamily {
    workspace
        .define_read_family("phase-four-registry-outcome", |read| {
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

fn relation(name: &str) -> RelationName {
    RelationName::new(name).expect("relation name should build")
}

fn equality(aspect: &str, field: &str, value: &str) -> EqualityPredicate {
    EqualityPredicate::new(
        aspect,
        field,
        WorthQueryPredicateOperand::string(value.to_string()),
    )
    .expect("equality predicate should build")
}

fn relation_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "graph-read-access-phase-four-relation",
        [SchemaFieldView::new(
            crate::facade::foundation::AspectName::new("identity")
                .expect("schema aspect literal must be valid"),
            crate::facade::foundation::FieldName::new("id")
                .expect("schema field literal must be valid"),
            ScalarAspectType::String,
        )],
        [SchemaRelationView::new(
            crate::facade::foundation::RelationName::new("manager")
                .expect("schema relation literal must be valid"),
            2,
        )],
    )
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
