pub use worth_foundational::facade::{AspectValue, ScalarAspectType};
pub use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
    ApplicationCapabilityAllowRule, ApplicationCapabilityCardinalityDimension,
    ApplicationCapabilityComposition, ApplicationCapabilityConstraintDefinition,
    ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
    ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationRule,
    ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
    ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
    ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRequirement,
    ApplicationCapabilityGraphRule, ApplicationCapabilityPropagationComposition,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
    ApplicationCapabilityScopeGuard, ApplicationCapabilitySeparationOfDutyRule,
    ApplicationCapabilityTargetDefinition, ApplicationCapabilityValidityDefinition,
    ApplicationCapabilityValidityTimeline, ApplicationCapabilityValueBinding,
    ApplicationCapabilityWorkflowDefinition, ErasedApplicationCapabilityContract,
    WorthQueryPortableApplicationCapabilityAcceptedValuesParts,
    WorthQueryPortableApplicationCapabilityConstraintParts,
    WorthQueryPortableApplicationCapabilityContractParts,
    WorthQueryPortableApplicationCapabilityDelegationParts,
    WorthQueryPortableApplicationCapabilityFieldBindingParts,
    WorthQueryPortableApplicationCapabilityGraphClauseParts,
    WorthQueryPortableApplicationCapabilityGraphRequirementParts,
    WorthQueryPortableApplicationCapabilityGraphRuleParts,
    WorthQueryPortableApplicationCapabilityRelationBindingParts,
    WorthQueryPortableApplicationCapabilityScopeGuardParts,
    WorthQueryPortableApplicationCapabilityValueBindingParts,
};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    WorthQueryPortableApplicationAuthorizationPathParts,
};
pub use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

pub fn currentness() -> ApplicationCapabilityCurrentnessDefinition {
    ApplicationCapabilityCurrentnessDefinition::new(
        value("Status", 1),
        ApplicationCapabilityWorkflowDefinition::new(field("Workflow"), field("Workflow")),
        ApplicationCapabilityValidityDefinition::new(
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
            field("NotBefore"),
            field("NotAfter"),
        ),
    )
}

pub fn delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityDelegationParts {
            parent: relation("Parent", "Grant", "Grant"),
            grantor: relation("Grantor", "Principal", "Grant"),
            grantee: relation("Grantee", "Principal", "Grant"),
            limit: field("Limit"),
            provenance: "Provenance".to_owned(),
            provenance_type: identity("worth.tests.provenance.v1"),
            activation: None,
            revocation: None,
        },
    )
}

pub fn field(name: &str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityFieldBindingParts {
            entity: "Grant".to_owned(),
            aspect: "Facts".to_owned(),
            field: name.to_owned(),
            scalar_family: ScalarAspectType::UInt64,
            value_type: "worth.rust.u64".to_owned(),
        },
    )
}

pub fn value(name: &str, value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityValueBindingParts {
            field: field(name),
            value: AspectValue::UInt64(value),
        },
    )
}

pub fn relation(name: &str, from: &str, to: &str) -> ApplicationCapabilityRelationBinding {
    ApplicationCapabilityRelationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityRelationBindingParts {
            relation: name.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
        },
    )
}

pub fn identity(value: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(value.to_owned())
}
