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
