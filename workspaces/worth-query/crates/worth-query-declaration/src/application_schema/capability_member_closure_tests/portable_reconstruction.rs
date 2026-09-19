use super::*;
use crate::application_capability::{
    application_capability_canonical_components, ApplicationCapabilityGraphRequirement,
    WorthQueryPortableApplicationCapabilityAcceptedValuesParts,
    WorthQueryPortableApplicationCapabilityGraphClauseParts,
    WorthQueryPortableApplicationCapabilityGraphRequirementParts,
    WorthQueryPortableApplicationCapabilityGraphRuleParts,
    WorthQueryPortableApplicationCapabilityScopeGuardParts,
};

#[test]
fn owned_capability_parts_round_trip_exact_meaning_and_fresh_closure() {
    let original = contract(false, false, true);
    let reconstructed = ErasedContract::from_untrusted_parts(original.parts());

    assert_eq!(reconstructed, original);
    assert_eq!(
        application_capability_canonical_components(&reconstructed),
        application_capability_canonical_components(&original)
    );
    assert_eq!(build_from_members(members(reconstructed)), Ok(()));
}

#[test]
fn forged_operation_identity_is_not_admitted_as_authored_capability_meaning() {
    let mut parts = contract(false, false, true).into_parts();
    parts.operation_type = crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
        "worth.tests.forged-operation.v1".to_owned(),
    );
    let reconstructed = ErasedContract::from_untrusted_parts(parts);

    assert_eq!(
        build_from_members(members(reconstructed)),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn noncanonical_accepted_values_are_preserved_then_denied() {
    let canonical = ApplicationCapabilityAcceptedValues::one_of(
        field::<Field>("Field"),
        [encoded(1_u64), encoded(2_u64)],
    );
    let mut parts = canonical.parts();
    parts.values.reverse();
    let requirement = ApplicationCapabilityAcceptedValues::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
            field: parts.field,
            values: parts.values,
        },
    );
    assert_eq!(requirement.values().len(), 2);
    assert!(requirement.values()[0] > requirement.values()[1]);

    let guard = ApplicationCapabilityScopeGuard::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityScopeGuardParts {
            requirements: vec![requirement],
        },
    );
    let composition =
        composition_with_disclosure(ApplicationCapabilityDisclosureRule::Permit(vec![guard]));

    assert_eq!(
        build_from_members(members(contract_with_composition(
            false,
            false,
            composition,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidCanonicalOrdering)
    );
}

#[test]
fn duplicate_graph_requirements_are_preserved_then_denied() {
    let base = composition(true);
    let requirement = base.decision().allow().graph().requirements()[0].clone();
    let allow = ApplicationCapabilityGraphRule::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRuleParts {
            requirements: vec![requirement.clone(), requirement],
        },
    );
    let composition = ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow),
            base.decision().deny().clone(),
            base.decision().conflict().clone(),
        ),
        base.actors().clone(),
        base.propagation().clone(),
    );

    assert_eq!(
        build_from_members(members(contract_with_composition(
            false,
            false,
            composition,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidCanonicalOrdering)
    );
}

#[test]
fn reconstructed_authorization_path_must_still_close_over_declared_topology() {
    let base = composition(true);
    let source_clause = &base.decision().allow().graph().requirements()[0].clauses()[0];
    let mut path_parts = source_clause.path().parts();
    path_parts.scope_entity = "Grant".to_owned();
    let clause = ApplicationCapabilityGraphClause::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphClauseParts {
            path: crate::application_schema::ApplicationAuthorizationPath::from_untrusted_parts(
                path_parts,
            ),
            guard: source_clause.guard().clone(),
            context_anchors: source_clause.context_anchors().to_vec(),
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
    let composition = ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(allow),
            base.decision().deny().clone(),
            base.decision().conflict().clone(),
        ),
        base.actors().clone(),
        base.propagation().clone(),
    );

    assert_eq!(
        build_from_members(members(contract_with_composition(
            false,
            false,
            composition,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

fn composition_with_disclosure(
    disclosure: ApplicationCapabilityDisclosureRule,
) -> ApplicationCapabilityComposition {
    let base = composition(true);
    ApplicationCapabilityComposition::new(
        base.decision().clone(),
        base.actors().clone(),
        ApplicationCapabilityPropagationComposition::new(
            base.propagation().delegation(),
            disclosure,
        ),
    )
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
