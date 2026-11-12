//! Pure declaration and closure evidence; installed authorization has its own consumer court.
use super::super::*;
use super::scope::ApplicationDecisionReadScopeMeaning as Meaning;

fn entity(name: &str) -> ApplicationOperationDecisionReadTarget {
    ApplicationOperationDecisionReadTarget::Entity {
        entity: name.to_owned(),
    }
}
fn scope(targets: Vec<ApplicationOperationDecisionReadTarget>) -> Meaning {
    Meaning::from_untrusted_parts(targets, Vec::new()).unwrap()
}
fn operation() -> ApplicationSchemaMember {
    ApplicationSchemaMember::Operation {
        operation: "Read".into(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "worth.rust.unit",
        ),
    }
}
fn reference(definition: &Meaning) -> ApplicationSchemaMember {
    ApplicationSchemaMember::OperationDecisionReadScope {
        operation: "Read".into(),
        scope: definition.identity().into(),
    }
}

#[test]
fn scope_identity_is_canonical_on_contents_and_insertion_order() {
    let forward = scope(vec![entity("A"), entity("B")]);
    let reversed = scope(vec![entity("B"), entity("A"), entity("A")]);
    assert_eq!(forward, reversed);
    assert_ne!(
        forward.identity(),
        scope(vec![entity("A"), entity("C")]).identity()
    );
}

#[test]
fn field_and_relation_scope_identity_retains_every_locus_axis() {
    let field =
        |entity: &str, aspect: &str, field: &str| ApplicationOperationDecisionReadTarget::Field {
            entity: entity.into(),
            aspect: aspect.into(),
            field: field.into(),
        };
    let baseline = scope(vec![field("A", "Facts", "value")]);
    for changed in [
        field("B", "Facts", "value"),
        field("A", "Other", "value"),
        field("A", "Facts", "other"),
    ] {
        assert_ne!(baseline.identity(), scope(vec![changed]).identity());
    }
    let relation =
        |name: &str, from: &str, to: &str| ApplicationOperationDecisionReadTarget::Relation {
            relation: name.into(),
            from: from.into(),
            to: to.into(),
        };
    let baseline = scope(vec![relation("Owns", "A", "B")]);
    for changed in [
        relation("Other", "A", "B"),
        relation("Owns", "B", "B"),
        relation("Owns", "A", "A"),
    ] {
        assert_ne!(baseline.identity(), scope(vec![changed]).identity());
    }
}

#[test]
fn renaming_a_scope_marker_preserves_its_schema_identity_and_contents() {
    struct First;
    struct Renamed;
    struct Entity;
    impl OperationReads<First> for Entity {}
    impl OperationReads<Renamed> for Entity {}
    let builder = || {
        ApplicationSchemaDeclarationBuilder::<()>::from_test_members(vec![
            ApplicationSchemaMember::Entity { entity: "A".into() },
        ])
    };
    let original = builder()
        .decision_read_scope(
            ApplicationDecisionReadScopeDefinition::<(), First>::new().entity(
                ApplicationEntityRef::<(), Entity>::from_schema_identifier("A"),
            ),
        )
        .build()
        .unwrap();
    let renamed = builder()
        .decision_read_scope(
            ApplicationDecisionReadScopeDefinition::<(), Renamed>::new().entity(
                ApplicationEntityRef::<(), Entity>::from_schema_identifier("A"),
            ),
        )
        .build()
        .unwrap();
    assert_eq!(original.identity(), renamed.identity());
    assert_eq!(original.erased().members(), renamed.erased().members());
}

#[test]
fn mixed_scope_and_inline_grants_have_exact_union_contents() {
    let definition = Meaning::from_untrusted_parts(vec![entity("A")], vec!["Owns".into()]).unwrap();
    let members = vec![
        reference(&definition),
        ApplicationSchemaMember::DecisionReadScope { definition },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "Read".into(),
            target: entity("B"),
        },
        ApplicationSchemaMember::OperationDerivedRead {
            operation: "Read".into(),
            relation: "Touches".into(),
        },
    ];
    let actual = application_operation_decision_read_targets(&members, "Read")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(actual, vec![entity("A"), entity("B")]);
    assert!(!actual.contains(&entity("Outside")));
    assert_eq!(
        application_operation_decision_read_targets(&members, "Other").count(),
        0
    );
    assert_eq!(
        application_operation_derived_read_relations(&members, "Read").collect::<Vec<_>>(),
        vec!["Owns", "Touches"]
    );
    assert_eq!(
        application_operation_derived_read_relations(&members, "Other").count(),
        0
    );
}

#[test]
fn shared_contents_still_require_their_real_declared_dependencies() {
    let definition = scope(vec![entity("Missing")]);
    let members = vec![
        operation(),
        reference(&definition),
        ApplicationSchemaMember::DecisionReadScope { definition },
    ];
    assert!(
        matches!(super::super::member_closure::validate_member_closure(&members),
        Err(ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { dependency: ApplicationOperationDependencyLocus::DecisionReadTarget(ApplicationOperationDecisionReadTarget::Entity { entity }), .. }) if entity == "Missing")
    );
}

#[test]
fn a_reference_without_its_contents_refuses_at_declaration_closure() {
    let members = vec![operation(), reference(&scope(vec![entity("A")]))];
    assert!(
        matches!(super::super::member_closure::validate_member_closure(&members),
        Err(ApplicationSchemaDeclarationDenial::MissingDecisionReadScope { operation, .. }) if operation == "Read")
    );
}

#[test]
fn scope_contents_cannot_authorize_an_undeclared_operation() {
    let definition = scope(vec![entity("A")]);
    let members = vec![
        ApplicationSchemaMember::Entity { entity: "A".into() },
        reference(&definition),
        ApplicationSchemaMember::DecisionReadScope { definition },
    ];
    assert!(matches!(
        super::super::member_closure::validate_member_closure(&members),
        Err(ApplicationSchemaDeclarationDenial::MissingDecisionReadScope { .. })
    ));
}

#[test]
fn a_shared_field_still_satisfies_only_its_exact_precondition_read() {
    let field = ApplicationOperationDecisionReadTarget::Field {
        entity: "A".into(),
        aspect: "Facts".into(),
        field: "value".into(),
    };
    let definition = scope(vec![field]);
    let mut members = vec![
        operation(),
        reference(&definition),
        ApplicationSchemaMember::DecisionReadScope { definition },
        ApplicationSchemaMember::Entity { entity: "A".into() },
        ApplicationSchemaMember::Aspect {
            entity: "A".into(),
            aspect: "Facts".into(),
            identity: AspectIdentity(1),
            revision: AspectContractRevision(1),
            absence: worth_foundational::facade::AbsenceLaw::Required,
        },
        ApplicationSchemaMember::Field {
            entity: "A".into(),
            aspect: "Facts".into(),
            field: "value".into(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: worth_foundational::facade::ScalarAspectType::UInt64,
            value_type: "worth.rust.u64".into(),
            canonical: None,
            content_format: None,
            unit: None,
            frame: None,
            writable: false,
            equality_queryable: false,
            unique: false,
        },
        ApplicationSchemaMember::OperationMutationPrecondition {
            operation: "Read".into(),
            target: ApplicationMutationPreconditionTarget::from_untrusted_fields(
                ApplicationMutationPreconditionFamily::ExpectedVersion,
                "A".into(),
                "Facts".into(),
                "value".into(),
            ),
        },
    ];
    assert_eq!(
        super::super::member_closure::validate_member_closure(&members),
        Ok(())
    );
    members.retain(|member| {
        !matches!(
            member,
            ApplicationSchemaMember::OperationDecisionReadScope { .. }
        )
    });
    assert!(matches!(
        super::super::member_closure::validate_member_closure(&members),
        Err(
            ApplicationSchemaDeclarationDenial::MissingOperationMutationPreconditionDependency { .. }
        )
    ));
}
