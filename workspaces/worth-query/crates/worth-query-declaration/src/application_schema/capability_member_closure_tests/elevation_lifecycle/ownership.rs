use super::*;
use crate::application_capability::{
    ApplicationCapabilityDelegationActivationDefinition, ApplicationCapabilityRevocationDefinition,
};

struct OtherCapability;

#[test]
fn one_lifecycle_operation_cannot_serve_two_governed_capability_owners() {
    let first = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::Distinct,
    );
    let ordinary = elevation_definition(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::Distinct,
        std::time::Duration::from_secs(20 * 60),
    );
    let aliased_lifecycle = ApplicationCapabilityElevationLifecycleDefinition::new(
        ordinary.lifecycle().elevation_slot().clone(),
        ordinary.lifecycle().review_slot().clone(),
        transition_binding::<RequestCapability, RequestOperation>("RequestCapability", "Request"),
        ordinary.lifecycle().approve().clone(),
        ordinary.lifecycle().revoke().clone(),
        ordinary.lifecycle().complete_review().clone(),
    );
    let aliased = ApplicationCapabilityElevationDefinition::new(
        ordinary.identity().clone(),
        ordinary.reason().clone(),
        ordinary.status().clone(),
        ordinary.states().clone(),
        ordinary.validity().clone(),
        ordinary.maximum_duration(),
        ordinary.requester().clone(),
        ordinary.approver().clone(),
        ordinary.grant().clone(),
        aliased_lifecycle,
        ordinary.review().clone(),
    );
    let second = ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, OtherCapability>::from_schema_identifier(
            "OtherCapability",
        ),
        operation::<Operation>("Operation"),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::governed(aliased))
    .build()
    .erased()
    .clone();
    let mut members = elevation_members(first);
    members.push(ApplicationSchemaMember::ApplicationCapability { contract: second });

    assert_eq!(
        build_from_members(members),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn lifecycle_operation_cannot_also_own_capability_revocation() {
    let contract = ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        operation::<Operation>("Operation"),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition().with_revocation(
        ApplicationCapabilityRevocationDefinition::new(
            operation::<RequestOperation>("Request"),
            binding::<Action>("Action"),
            ApplicationCapabilityValueBinding::new(field::<Status>("Status"), encoded(2_u64)),
        ),
    ))
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::governed(
        elevation_definition(
            StatePosture::Distinct,
            ReviewPosture::Distinct,
            LifecyclePosture::Distinct,
            std::time::Duration::from_secs(20 * 60),
        ),
    ))
    .build()
    .erased()
    .clone();
    assert_eq!(
        build_from_members(elevation_members(contract)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn lifecycle_operation_cannot_also_own_delegation_activation() {
    let contract = ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        operation::<Operation>("Operation"),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition().with_activation(
        ApplicationCapabilityDelegationActivationDefinition::new(
            operation::<RequestOperation>("Request"),
            binding::<Action>("Action"),
        ),
    ))
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::governed(
        elevation_definition(
            StatePosture::Distinct,
            ReviewPosture::Distinct,
            LifecyclePosture::Distinct,
            std::time::Duration::from_secs(20 * 60),
        ),
    ))
    .build()
    .erased()
    .clone();
    assert_eq!(
        build_from_members(elevation_members(contract)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
