use super::*;

#[test]
fn declared_context_anchor_closes_over_its_exact_path_traversal() {
    let anchor = ApplicationCapabilityPathContextAnchor::after_forward(
        ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
            from_schema_identifiers(
                "PrincipalResource", "Principal", "Resource",
                crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
        resource_slot::<Context, ResourceSlot>("Context", "ResourceSlot"),
    );
    let contract = contract_with_composition(false, false, anchored_composition(anchor));
    assert_eq!(build_from_members(members(contract)), Ok(()));
}

#[test]
fn context_anchor_rejects_absent_traversal_and_undeclared_slot() {
    let absent_traversal = ApplicationCapabilityPathContextAnchor::after_forward(
        ApplicationRelationRef::<Schema, ScopedRelation, Grant, Resource>::from_schema_identifiers(
            "ScopedRelation",
            "Grant",
            "Resource",
            crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        ),
        resource_slot::<Context, ResourceSlot>("Context", "ResourceSlot"),
    );
    let contract = contract_with_composition(false, false, anchored_composition(absent_traversal));
    assert_eq!(
        build_from_members(members(contract)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );

    let missing_slot = ApplicationCapabilityPathContextAnchor::after_forward(
        ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
            from_schema_identifiers(
                "PrincipalResource", "Principal", "Resource",
                crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
        resource_slot::<Context, MissingResourceSlot>("Context", "MissingResourceSlot"),
    );
    let contract = contract_with_composition(false, false, anchored_composition(missing_slot));
    assert_eq!(
        build_from_members(members(contract)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn context_anchor_cannot_cross_the_contract_context() {
    let anchor = ApplicationCapabilityPathContextAnchor::after_forward(
        ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
            from_schema_identifiers(
                "PrincipalResource", "Principal", "Resource",
                crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            ),
        resource_slot::<OtherContext, OtherResourceSlot>("OtherContext", "OtherResourceSlot"),
    );
    let contract = contract_with_composition(false, false, anchored_composition(anchor));
    let mut members = members(contract);
    members.push(ApplicationSchemaMember::ApplicationCapabilityContext {
        context: "OtherContext".to_string(),
        context_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "OtherContext",
        ),
    });
    members.push(
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: "OtherContext".to_string(),
            context_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "OtherContext",
            ),
            slot: "OtherResourceSlot".to_string(),
            slot_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "OtherResourceSlot",
            ),
            entity: "Resource".to_string(),
        },
    );
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}
