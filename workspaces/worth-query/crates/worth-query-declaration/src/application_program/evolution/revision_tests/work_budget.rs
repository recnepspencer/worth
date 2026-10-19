//! Program meaning larger than the declared canonical work budget is denied,
//! never hashed silently and never a panic.

use super::{BaselineProgram, BoundedFeature, RevisionSchema, SHARED_IDENTITY};
use crate::application_program::program::deny_canonical_revision_budget;
use crate::application_program::{
    ApplicationFeatureSpec, ApplicationProgramDefinition, ApplicationProgramManifest,
    ApplicationProgramRevision, ApplicationProgramRevisionBudgetDenial,
    ApplicationProgramValidationDenialKind, CanonicalDigestWorkBudget,
};
const DEFAULT: CanonicalDigestWorkBudget =
    <BaselineProgram as ApplicationProgramDefinition<RevisionSchema>>::CANONICAL_REVISION_BUDGET;

/// One feature declaration contributes one manifest record, so this many
/// declarations put the basis past the declared 4_096-entry ceiling.
const OVER_BUDGET_FEATURE_COUNT: usize = 5_000;

#[test]
fn a_manifest_past_the_declared_entry_budget_is_denied_rather_than_hashed() {
    let denial = ApplicationProgramRevision::mint(&over_budget_manifest(), DEFAULT)
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
    let budget = ApplicationProgramRevision::mint(&over_budget_manifest(), DEFAULT)
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
    assert!(ApplicationProgramRevision::mint(
        &ApplicationProgramManifest::normalize(
            SHARED_IDENTITY.as_str(),
            &[declared_bounded_feature()],
            &[],
            &[],
            &[],
            None,
        ),
        DEFAULT
    )
    .is_ok());
}

/// Normalizes a manifest that no host would author but any host could present:
/// the same declared feature repeated past the canonical entry ceiling.
fn over_budget_manifest() -> ApplicationProgramManifest {
    let features = vec![declared_bounded_feature(); OVER_BUDGET_FEATURE_COUNT];
    ApplicationProgramManifest::normalize(SHARED_IDENTITY.as_str(), &features, &[], &[], &[], None)
}

fn declared_bounded_feature() -> crate::application_program::ApplicationFeatureDeclaration {
    let (feature, _actions) = ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
        .finish()
        .into_parts();
    feature
}

struct Budgeted<const BYTES: usize>;
impl<const BYTES: usize> ApplicationProgramDefinition<RevisionSchema> for Budgeted<BYTES> {
    type Contributions =
        <BaselineProgram as ApplicationProgramDefinition<RevisionSchema>>::Contributions;
    type Outputs = <BaselineProgram as ApplicationProgramDefinition<RevisionSchema>>::Outputs;
    type Rules = <BaselineProgram as ApplicationProgramDefinition<RevisionSchema>>::Rules;
    const IDENTITY: crate::application_program::ApplicationProgramIdentity = SHARED_IDENTITY;
    const CANONICAL_REVISION_BUDGET: CanonicalDigestWorkBudget =
        match CanonicalDigestWorkBudget::new(4096, BYTES) {
            Some(budget) => budget,
            None => panic!("finite nonzero test budget"),
        };
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        BaselineProgram::feature_specs()
    }
}

#[test]
fn a_programs_declared_budget_is_enforced_without_changing_accepted_meaning() {
    use crate::application_program::ApplicationProgramAuthoring;
    let denied =
        ApplicationProgramAuthoring::<RevisionSchema, Budgeted<1>>::begin().validated_program();
    assert!(matches!(denied, Err(cause)
        if cause.kind() == ApplicationProgramValidationDenialKind::CanonicalRevisionBudgetExceeded));
    assert_eq!(
        super::revision_of::<Budgeted<{ 1024 * 1024 }>>(),
        super::revision_of::<Budgeted<{ 2 * 1024 * 1024 }>>(),
    );
    assert_eq!(
        super::revision_of::<BaselineProgram>(),
        super::revision_of::<Budgeted<{ 2 * 1024 * 1024 }>>(),
    );
}
