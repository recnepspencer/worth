use worth_foundational::facade::{
    compare_canonical_basis, prepare_canonical_comparison, AspectValue, CanonicalComparisonOutcome,
    CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, CanonicalEquivalenceBasis,
    InternedString,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::{account_parameter, installed_query};
use crate::application_query::admit_application_query_parameters;
use crate::application_query::parameter_canonical_basis::prepare_parameter_basis;

#[test]
fn parameter_bindings_converge_and_diverge_through_foundational_comparison() {
    let query = installed_query();
    let left = admitted(&query, 7);
    let equivalent = admitted(&query, 7);
    let changed = admitted(&query, 8);

    assert!(left
        .canonical_basis()
        .is_equivalent_to(equivalent.canonical_basis()));
    assert!(!left
        .canonical_basis()
        .is_equivalent_to(changed.canonical_basis()));
    let equivalent = compare(left, equivalent);
    assert!(matches!(
        equivalent,
        CanonicalComparisonOutcome::Equivalent(_)
    ));
    let changed = compare(left_again(&query), changed);
    assert!(matches!(changed, CanonicalComparisonOutcome::Mismatched(_)));
}

#[test]
fn parameter_identity_has_no_debug_or_precanonical_value_grammar() {
    let canonical_source = include_str!("../parameter_canonical_basis.rs");
    let source = include_str!("../parameter_binding.rs");
    assert!(!source.contains("{:?}"));
    assert!(!source.contains("prepare_aspect_value_identity_basis"));
    assert!(!source.contains("worth_query_admitted_application_parameters_v1"));
    assert!(!canonical_source.contains("admission_digest"));
    assert!(!canonical_source.contains("hash_parts"));
    assert!(canonical_source.contains("CanonicalDigestAlgorithmId::sha256()"));
}

#[test]
fn parameter_canonicalization_denies_entry_and_encoded_byte_overflow() {
    let value = AspectValue::String(InternedString::Raw("x".repeat(128)));
    let bindings = [("message", value)];
    let entry_denial =
        prepare_parameter_basis(&bindings, CanonicalDigestWorkBudget::new(3, 4096).unwrap())
            .unwrap_err();
    assert!(matches!(
        entry_denial,
        CanonicalDigestDerivationDenial::EntryLimitExceeded {
            maximum: 3,
            actual: 4
        }
    ));

    let byte_denial =
        prepare_parameter_basis(&bindings, CanonicalDigestWorkBudget::new(4, 64).unwrap())
            .unwrap_err();
    assert!(matches!(
        byte_denial,
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { maximum: 64, .. }
    ));
}

fn admitted(
    query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
        super::PlanningTestSchema,
        super::ActivityQuery,
        super::ActivityParameters,
        super::ActivityResult,
        super::Account,
    >,
    account: u64,
) -> crate::application_query::WorthQueryAdmittedApplicationQueryParameters {
    admit_application_query_parameters(
        query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), account)
            .unwrap(),
    )
    .unwrap()
}

fn left_again(
    query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
        super::PlanningTestSchema,
        super::ActivityQuery,
        super::ActivityParameters,
        super::ActivityResult,
        super::Account,
    >,
) -> crate::application_query::WorthQueryAdmittedApplicationQueryParameters {
    admitted(query, 7)
}

fn compare(
    left: crate::application_query::WorthQueryAdmittedApplicationQueryParameters,
    right: crate::application_query::WorthQueryAdmittedApplicationQueryParameters,
) -> CanonicalComparisonOutcome {
    let ready = prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left.canonical_basis().basis().clone(),
        right.canonical_basis().basis().clone(),
    )
    .into_result()
    .expect("prepared parameter bases admit exact comparison");
    compare_canonical_basis(&ready)
}
