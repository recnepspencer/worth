use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_capability::{
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
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    WorthQueryPortableApplicationAuthorizationPathParts,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[test]
fn downstream_decoder_constructs_owned_capability_meaning_without_typed_references() {
    let field = field("Field");
    let accepted = ApplicationCapabilityAcceptedValues::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
            field: field.clone(),
            values: vec![AspectValue::UInt64(2), AspectValue::UInt64(1)],
        },
    );
    let guard = ApplicationCapabilityScopeGuard::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityScopeGuardParts {
            requirements: vec![accepted],
        },
    );
    let path = ApplicationAuthorizationPath::from_untrusted_parts(
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: ApplicationAuthorizationPathEffect::Allow,
            principal_entity: "Principal".to_owned(),
            scope_entity: "Resource".to_owned(),
            traversals: Vec::new(),
            predicates: Vec::new(),
        },
    );
    let clause = ApplicationCapabilityGraphClause::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphClauseParts {
            path,
            guard: ApplicationCapabilityScopeGuard::unconditional(),
            context_anchors: Vec::new(),
        },
    );
    let requirement = ApplicationCapabilityGraphRequirement::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRequirementParts {
            clauses: vec![clause],
        },
    );
    let allow = ApplicationCapabilityGraphRule::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRuleParts {
            requirements: vec![requirement],
        },
    );
    let contract = ErasedApplicationCapabilityContract::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityContractParts {
            name: "Capability".to_owned(),
            capability_type: identity("worth.tests.capability.v1"),
            operation: "Operate".to_owned(),
            operation_type: identity("Operate"),
            input_type: identity("worth.rust.unit"),
            grant_entity: "Grant".to_owned(),
            target: ApplicationCapabilityTargetDefinition::new(
                value("Action", 1),
                relation("Resource", "Grant", "Resource"),
                ApplicationCapabilityRelationDimension::NotApplicable,
                ApplicationCapabilityFieldDimension::Bound(field.clone()),
                value("Purpose", 1),
            ),
            constraints: ApplicationCapabilityConstraintDefinition::from_untrusted_parts(
                WorthQueryPortableApplicationCapabilityConstraintParts {
                    magnitude: ApplicationCapabilityFieldDimension::NotApplicable,
                    cardinality: ApplicationCapabilityCardinalityDimension::One,
                    currentness: currentness(),
                    context: "Context".to_owned(),
                    context_type: identity("worth.tests.context.v1"),
                },
            ),
            delegation: delegation(),
            composition: ApplicationCapabilityComposition::new(
                ApplicationCapabilityDecisionComposition::new(
                    ApplicationCapabilityAllowRule::new(allow),
                    ApplicationCapabilityDenyRule::not_applicable(),
                    worth_query_declaration::facade::application_capability::ApplicationCapabilityConflictRule::not_applicable(),
                ),
                ApplicationCapabilityActorComposition::new(
                    ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                    ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
                ApplicationCapabilityPropagationComposition::new(
                    ApplicationCapabilityDelegationRule::forbidden(),
                    ApplicationCapabilityDisclosureRule::Permit(vec![guard]),
                ),
            ),
            elevation: ApplicationCapabilityElevationRule::not_applicable().parts(),
        },
    );

    assert_eq!(contract.name(), "Capability");
    assert_eq!(
        contract
            .composition()
            .propagation()
            .disclosure()
            .guards()
            .unwrap()[0]
            .requirements()[0]
            .values(),
        &[AspectValue::UInt64(2), AspectValue::UInt64(1)]
    );
    assert_eq!(
        ErasedApplicationCapabilityContract::from_untrusted_parts(contract.parts()),
        contract
    );
}

fn currentness() -> ApplicationCapabilityCurrentnessDefinition {
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

fn delegation() -> ApplicationCapabilityDelegationDefinition {
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

fn field(name: &str) -> ApplicationCapabilityFieldBinding {
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

fn value(name: &str, value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityValueBindingParts {
            field: field(name),
            value: AspectValue::UInt64(value),
        },
    )
}

fn relation(name: &str, from: &str, to: &str) -> ApplicationCapabilityRelationBinding {
    ApplicationCapabilityRelationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityRelationBindingParts {
            relation: name.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
        },
    )
}

fn identity(value: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(value.to_owned())
}
