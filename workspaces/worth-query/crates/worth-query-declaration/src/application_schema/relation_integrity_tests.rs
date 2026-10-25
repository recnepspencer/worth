use super::{
    ApplicationRelationCardinality, ApplicationRelationDeletionPolicy,
    ApplicationRelationEndpoints, ApplicationRelationIntegrity,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
    ApplicationSchemaMember,
};

struct StrictSchema;
struct StrictEntity;

crate::worth_query_relation!(
    NoSelfPeer in StrictSchema,
    StrictEntity => StrictEntity; integrity = same_context_no_self_edges_unbounded_retain_dangling
);

fn members(integrity: ApplicationRelationIntegrity) -> Vec<ApplicationSchemaMember> {
    vec![
        ApplicationSchemaMember::Entity {
            entity: "From".into(),
        },
        ApplicationSchemaMember::Entity {
            entity: "To".into(),
        },
        ApplicationSchemaMember::Relation {
            relation: "Contains".into(),
            from: "From".into(),
            to: "To".into(),
            integrity,
        },
    ]
}

#[test]
fn relation_declaration_retains_its_explicit_integrity_contract() {
    let integrity = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let declaration =
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity))
            .build()
            .unwrap();

    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::Relation { integrity: actual, .. } if *actual == integrity
    )));
}

#[test]
fn invalid_declared_relation_cardinality_fails_before_execution() {
    let integrity = ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::same_context(false),
        ApplicationRelationCardinality::new(None, None, None, None, Some(2), Some(1)),
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
    );

    assert!(matches!(
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity)).build(),
        Err(ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity)
    ));
}

#[test]
fn relation_integrity_changes_canonical_schema_identity() {
    let retained = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let cascaded = ApplicationRelationIntegrity::new(
        retained.endpoints,
        retained.cardinality,
        ApplicationRelationDeletionPolicy::CascadeDeleteRelations,
    );
    let retained = ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(retained))
        .build()
        .unwrap();
    let cascaded = ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(cascaded))
        .build()
        .unwrap();

    assert_ne!(retained.identity(), cascaded.identity());
}

#[test]
fn no_self_edges_is_an_explicit_stronger_contract() {
    let established = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let stronger =
        ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling();

    assert!(established.endpoints.self_edges_allowed);
    assert!(!stronger.endpoints.self_edges_allowed);
    assert_eq!(established.cardinality, stronger.cardinality);
    assert_eq!(established.deletion, stronger.deletion);
    assert_ne!(established, stronger);
    assert_eq!(NoSelfPeer::reference().integrity(), stronger);
}

#[test]
fn adding_and_removing_acyclicity_changes_only_the_declared_meaning() {
    let unrestricted = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let acyclic = unrestricted.acyclic();
    let mut removed = acyclic;
    removed.acyclic = false;
    let declare = |integrity| {
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity))
            .build()
            .unwrap()
    };
    let original = declare(unrestricted);
    let strengthened = declare(acyclic);
    let restored = declare(removed);
    assert_ne!(original.identity(), strengthened.identity());
    assert_eq!(original.identity(), restored.identity());
    assert_eq!(acyclic.endpoints, unrestricted.endpoints);
    assert_eq!(acyclic.cardinality, unrestricted.cardinality);
    assert_eq!(acyclic.deletion, unrestricted.deletion);
}

#[test]
fn symmetry_and_each_uniqueness_scope_have_distinct_removable_meaning() {
    use super::ApplicationRelationUniquenessScope as Scope;
    let original = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let symmetric = original.symmetric();
    let directed = symmetric.unique(Scope::DirectedSemanticEdge);
    let normalized = symmetric.unique(Scope::NormalizedSymmetricEdge);
    let identity = |integrity| {
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity))
            .build()
            .unwrap()
            .identity()
            .clone()
    };
    assert_ne!(identity(original), identity(symmetric));
    assert_ne!(identity(symmetric), identity(directed));
    assert_ne!(identity(directed), identity(normalized));
    let mut removed = directed;
    removed.symmetric = false;
    removed.unique = None;
    assert_eq!(identity(original), identity(removed));
    assert_eq!(directed.endpoints, original.endpoints);
    assert_eq!(directed.cardinality, original.cardinality);
    assert_eq!(directed.deletion, original.deletion);
}

#[test]
fn partition_law_changes_meaning_without_changing_endpoint_permission() {
    let original = ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(
            true,
            super::ApplicationRelationCrossContextPolicy::AllowExplicit,
        ),
        ApplicationRelationCardinality::unbounded(),
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
    );
    let isolated = original.same_partition_endpoints();
    assert_eq!(isolated.endpoints, original.endpoints);
    let identity = |integrity| {
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity))
            .build()
            .unwrap()
            .identity()
            .clone()
    };
    assert_ne!(identity(original), identity(isolated));
    let mut removed = isolated;
    removed.same_partition_endpoints = false;
    assert_eq!(identity(original), identity(removed));
}

#[test]
fn minimum_kind_set_is_canonical_and_count_is_part_of_meaning() {
    let original = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let identity = |integrity| {
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(integrity))
            .build()
            .unwrap()
            .identity()
            .clone()
    };
    let two = original.reaches_at_least(2, &["To", "From", "To"]);
    assert_eq!(
        identity(two),
        identity(original.reaches_at_least(2, &["From", "To"]))
    );
    assert_ne!(
        identity(two),
        identity(original.reaches_at_least(3, &["From", "To"]))
    );
    assert_ne!(
        identity(two),
        identity(original.reaches_at_least(2, &["To"]))
    );
    let mut removed = two;
    removed.reaches_at_least = None;
    assert_eq!(identity(original), identity(removed));
}

#[test]
fn minimum_requires_a_positive_count_and_declared_nonempty_target_kinds() {
    let original = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    for invalid in [
        original.reaches_at_least(0, &["To"]),
        original.reaches_at_least(1, &[]),
        original.reaches_at_least(1, &["Unknown"]),
    ] {
        assert!(matches!(
            ApplicationSchemaDeclarationBuilder::<()>::from_test_members(members(invalid)).build(),
            Err(ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity)
        ));
    }
}
