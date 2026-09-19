//! Program meaning larger than the declared canonical work budget is denied,
//! never hashed silently and never a panic.

use super::{BoundedFeature, RevisionSchema, SHARED_IDENTITY};
use crate::application_program::program::deny_canonical_revision_budget;
use crate::application_program::{
    ApplicationFeatureSpec, ApplicationProgramManifest, ApplicationProgramRevision,
    ApplicationProgramRevisionBudgetDenial, ApplicationProgramValidationDenialKind,
};

/// One feature declaration contributes one manifest record, so this many
/// declarations put the basis past the declared 4_096-entry ceiling.
const OVER_BUDGET_FEATURE_COUNT: usize = 5_000;

#[test]
fn a_manifest_past_the_declared_entry_budget_is_denied_rather_than_hashed() {
    let denial = ApplicationProgramRevision::mint(&over_budget_manifest())
        .expect_err("meaning past the declared canonical budget cannot be identified");

    match denial {
        ApplicationProgramRevisionBudgetDenial::BasisEntries { maximum, attempted } => {
            assert_eq!(maximum, 4_096);
            assert!(u64::from(attempted) > u64::from(maximum));
        }
        ApplicationProgramRevisionBudgetDenial::EncodedBytes { .. } => {
            panic!("the entry ceiling is reached before the encoded-byte ceiling")
        }
    }
}

#[test]
fn an_exceeded_revision_budget_becomes_a_typed_program_validation_denial() {
    let budget = ApplicationProgramRevision::mint(&over_budget_manifest())
        .expect_err("meaning past the declared canonical budget cannot be identified");

    let denial = deny_canonical_revision_budget(&SHARED_IDENTITY, budget);

    assert_eq!(
        denial.kind(),
        ApplicationProgramValidationDenialKind::CanonicalRevisionBudgetExceeded
    );
    assert!(denial.subject().starts_with(SHARED_IDENTITY.as_str()));
    assert!(denial
        .subject()
        .contains("exceed the declared maximum 4096"));
}

#[test]
fn an_ordinary_program_manifest_stays_inside_the_declared_budget() {
    assert!(
        ApplicationProgramRevision::mint(&ApplicationProgramManifest::normalize(
            SHARED_IDENTITY.as_str(),
            &[declared_bounded_feature()],
            &[],
            &[],
            &[],
        ))
        .is_ok()
    );
}

/// Normalizes a manifest that no host would author but any host could present:
/// the same declared feature repeated past the canonical entry ceiling.
fn over_budget_manifest() -> ApplicationProgramManifest {
    let features = vec![declared_bounded_feature(); OVER_BUDGET_FEATURE_COUNT];
    ApplicationProgramManifest::normalize(SHARED_IDENTITY.as_str(), &features, &[], &[], &[])
}

fn declared_bounded_feature() -> crate::application_program::ApplicationFeatureDeclaration {
    let (feature, _actions) = ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
        .finish()
        .into_parts();
    feature
}
