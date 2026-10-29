pub(crate) use worth_query_worlds::facade::installation::installation_contracts::untrusted_application_capability_construction::*;

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
