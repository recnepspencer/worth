use super::*;

#[test]
fn capability_contract_requires_every_graph_rule_relation() {
    let mut members = members(contract(false, false, true));
    members.retain(|member| {
        !matches!(
            member,
            ApplicationSchemaMember::Relation { relation, .. }
                if relation == "PrincipalResource"
        )
    });
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn capability_resource_must_begin_at_the_declared_grant_entity() {
    assert_eq!(
        build_from_members(members(contract(true, false, true))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn capability_names_are_unique_even_when_other_meaning_differs() {
    let mut members = members(contract(false, false, true));
    members.push(ApplicationSchemaMember::ApplicationCapability {
        contract: contract(false, true, true),
    });
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::DuplicateApplicationCapability)
    );
}

#[test]
fn one_capability_name_cannot_claim_two_portable_identities() {
    let contracts = [
        contract_with_identity_and_composition(
            "Capability",
            "worth.query.test.first-capability.v1",
            false,
            false,
            composition(true),
        ),
        contract_with_identity_and_composition(
            "Capability",
            "worth.query.test.second-capability.v1",
            false,
            false,
            composition(true),
        ),
    ];
    let members = contracts
        .into_iter()
        .map(|contract| ApplicationSchemaMember::ApplicationCapability { contract })
        .collect();
    assert_eq!(
        ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
            .build()
            .map(|_| ()),
        Err(ApplicationSchemaDeclarationDenial::DuplicateMember)
    );
}

#[test]
fn capability_markers_cannot_alias_one_portable_identity() {
    let contracts = [
        contract_with_identity_and_composition(
            "FirstCapability",
            "worth.query.test.shared-capability.v1",
            false,
            false,
            composition(true),
        ),
        contract_with_identity_and_composition(
            "SecondCapability",
            "worth.query.test.shared-capability.v1",
            false,
            false,
            composition(true),
        ),
    ];
    let members = contracts
        .into_iter()
        .map(|contract| ApplicationSchemaMember::ApplicationCapability { contract })
        .collect::<Vec<_>>();
    assert_eq!(
        ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(members)
            .build()
            .map(|_| ()),
        Err(ApplicationSchemaDeclarationDenial::DuplicateMember)
    );
}

#[test]
fn blank_capability_identity_is_rejected_before_canonicalization() {
    let contract =
        contract_with_identity_and_composition("Capability", "", false, false, composition(true));
    assert_eq!(
        ApplicationSchemaDeclarationBuilder::<Schema>::from_test_members(vec![
            ApplicationSchemaMember::ApplicationCapability { contract },
        ])
        .build()
        .map(|_| ()),
        Err(ApplicationSchemaDeclarationDenial::InvalidIdentifier)
    );
}

#[test]
fn capability_portable_identity_preserves_schema_meaning_across_a_module_move() {
    mod original_location {
        crate::worth_query_capability!(
            pub(super) Capability in super::Schema,
            identity "worth.query.test.moved-capability.v1"
        );
    }
    mod moved_location {
        crate::worth_query_capability!(
            pub(super) Capability in super::Schema,
            identity "worth.query.test.moved-capability.v1"
        );
    }

    let mut original_members = members(contract_with_capability_ref(
        original_location::Capability::reference(),
    ));
    let mut moved_members = members(contract_with_capability_ref(
        moved_location::Capability::reference(),
    ));
    original_members.sort();
    moved_members.sort();
    let header = || super::super::canonical_identity::ApplicationSchemaCanonicalHeader {
        owner: "WORTH.tests",
        name: "module-move-capability",
        major: 1,
        minor: 0,
    };
    let original =
        super::super::canonical_identity::canonical_identity(header(), &original_members, &[]);
    let moved = super::super::canonical_identity::canonical_identity(header(), &moved_members, &[]);

    assert_eq!(original, moved);
    assert_ne!(
        std::any::type_name::<original_location::Capability>(),
        std::any::type_name::<moved_location::Capability>()
    );
}

#[test]
fn field_bound_capability_requires_an_explicit_disclosure_contract() {
    assert_eq!(
        build_from_members(members(contract(false, false, false))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn capability_currentness_requires_status_and_resource_workflow_fields() {
    for missing in ["Status", "ResourceWorkflow"] {
        let mut members = members(contract(false, false, true));
        members.retain(|member| {
            !matches!(
                member,
                ApplicationSchemaMember::Field { field, .. } if field == missing
            )
        });
        assert_eq!(
            build_from_members(members),
            Err(ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityDependency),
            "{missing} must remain an installed currentness dependency"
        );
    }
}

#[test]
fn capability_validity_timeline_requires_compatible_field_families() {
    let mut members = members(contract(false, false, true));
    for member in &mut members {
        if let ApplicationSchemaMember::Field {
            field,
            scalar_family,
            ..
        } = member
        {
            if field == "ValidFrom" {
                *scalar_family = ScalarAspectType::Int64;
            }
        }
    }
    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityDependency)
    );
}
