use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use super::registration_examples::InvariantRegistrationContract;
use super::{InvariantCatalog, InvariantRegistration};
use crate::validation::data::{InvariantExecutionPoint, InvariantFailureEffect, InvariantRule};

#[test]
fn canonical_catalog_digest_is_order_and_duplicate_independent_without_json_identity() {
    let unique_name =
        InvariantRegistration::commit_boundary_blocking(InvariantRule::unique_entity_aspect_field(
            crate::tests::support::aspect_key("profile.name"),
            crate::tests::support::field_key("value"),
        ));
    let max_merged = InvariantRegistration::harness_audit_only(InvariantRule::MaxMergedIntents(2));

    let left = InvariantCatalog {
        registrations: vec![unique_name.clone(), max_merged.clone(), unique_name.clone()],
    };
    let right = InvariantCatalog {
        registrations: vec![max_merged, unique_name],
    };

    assert_eq!(
        left.canonical_registration_digest(),
        right.canonical_registration_digest()
    );
    assert_eq!(left.canonicalized().registrations.len(), 2);
}

#[test]
fn catalog_digest_preserves_foundational_aspect_field_locator_identity() {
    let profile_name = catalog_for_unique_field(
        crate::tests::support::aspect_key("profile.name"),
        crate::tests::support::field_key("value"),
    );
    let profile_value = catalog_for_unique_field(
        crate::tests::support::aspect_key("profile.value"),
        crate::tests::support::field_key("name"),
    );

    assert_ne!(
        profile_name.canonical_registration_digest(),
        profile_value.canonical_registration_digest()
    );
}

#[test]
fn every_invariant_variant_has_a_registration_contract() {
    let catalog = InvariantCatalog::default();

    for rule in InvariantRule::registration_examples() {
        match rule.registration_contract() {
            InvariantRegistrationContract::DefaultAlwaysOnStructural => {
                assert!(
                    catalog.contains_registration_kind(&rule),
                    "default invariant rule {:?} is not registered in the default catalog",
                    rule
                );
            }
            InvariantRegistrationContract::OptInUserCatalog => {
                assert!(
                    !catalog.contains_registration_kind(&rule),
                    "opt-in invariant rule {:?} should not be silently pre-registered",
                    rule
                );
            }
        }
    }
}

#[test]
fn every_invariant_variant_supports_at_least_one_execution_point_and_can_register() {
    let execution_points = [
        InvariantExecutionPoint::MutationSensitive,
        InvariantExecutionPoint::CommitBoundary,
        InvariantExecutionPoint::SnapshotPublication,
        InvariantExecutionPoint::CertificationBoundary,
        InvariantExecutionPoint::HarnessAudit,
        InvariantExecutionPoint::GraphComposition,
    ];

    for rule in InvariantRule::registration_examples() {
        let supported_points = execution_points
            .into_iter()
            .filter(|point| rule.supports_execution_point(*point))
            .collect::<Vec<_>>();
        assert!(
            !supported_points.is_empty(),
            "invariant rule {:?} does not support any execution point",
            rule
        );
        for point in supported_points {
            let failure_effect = effect_supported_by_execution_point(point);
            let registration = InvariantRegistration::for_rule(rule.clone(), point, failure_effect);
            assert_eq!(registration.execution_point, point);
            assert_eq!(registration.rule, rule);
        }
    }
}

fn catalog_for_unique_field(aspect_key: AspectKey, field_key: FieldKey) -> InvariantCatalog {
    InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field_locator(AspectFieldLocator::new(
                LocatorAuthority::Planned,
                aspect_key,
                CanonicalFieldPath::single(field_key),
            )),
        )],
    }
}

fn effect_supported_by_execution_point(point: InvariantExecutionPoint) -> InvariantFailureEffect {
    match point {
        InvariantExecutionPoint::ProposedState
        | InvariantExecutionPoint::MutationSensitive
        | InvariantExecutionPoint::CommitBoundary
        | InvariantExecutionPoint::GraphComposition
        | InvariantExecutionPoint::HarnessAudit => InvariantFailureEffect::BlockCommit,
        InvariantExecutionPoint::SnapshotPublication
        | InvariantExecutionPoint::CertificationBoundary => {
            InvariantFailureEffect::BlockPublication
        }
    }
}
