use super::emission::{lifecycle_contract, EffectPosture, ResourceRelationPosture};
use super::{elevation_contract, LifecyclePosture, ReviewPosture, StatePosture};
use crate::application_capability::{
    application_capability_canonical_components, ErasedApplicationCapabilityContract,
};
use crate::application_schema::canonical_identity::{
    canonical_identity, ApplicationSchemaCanonicalHeader,
};
use crate::application_schema::{ApplicationSchemaIdentity, ApplicationSchemaMember};

#[test]
fn swapping_lifecycle_operation_roles_changes_canonical_identity() {
    let ordinary = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::Distinct,
    );
    let swapped = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::SwappedOperations,
    );
    assert_ne!(
        application_capability_canonical_components(&ordinary),
        application_capability_canonical_components(&swapped)
    );
}

#[test]
fn lifecycle_effect_binding_changes_capability_and_schema_identity() {
    assert_contract_and_schema_identity_differ(
        lifecycle_contract(
            EffectPosture::NotApplicable,
            ResourceRelationPosture::Governed,
        ),
        lifecycle_contract(EffectPosture::Derived, ResourceRelationPosture::Governed),
    );
}

#[test]
fn lifecycle_resource_relation_changes_capability_and_schema_identity() {
    assert_contract_and_schema_identity_differ(
        lifecycle_contract(
            EffectPosture::NotApplicable,
            ResourceRelationPosture::NotApplicable,
        ),
        lifecycle_contract(
            EffectPosture::NotApplicable,
            ResourceRelationPosture::Governed,
        ),
    );
}

fn assert_contract_and_schema_identity_differ(
    ordinary: ErasedApplicationCapabilityContract,
    changed: ErasedApplicationCapabilityContract,
) {
    assert_ne!(
        application_capability_canonical_components(&ordinary),
        application_capability_canonical_components(&changed)
    );
    assert_ne!(schema_identity(ordinary), schema_identity(changed));
}

fn schema_identity(contract: ErasedApplicationCapabilityContract) -> ApplicationSchemaIdentity {
    canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "canonical-twin-owner",
            name: "canonical-twin-schema",
            major: 1,
            minor: 0,
        },
        &[ApplicationSchemaMember::ApplicationCapability { contract }],
        &[],
    )
}
