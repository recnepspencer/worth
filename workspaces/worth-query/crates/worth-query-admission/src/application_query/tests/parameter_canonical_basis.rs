use worth_foundational::facade::{
    compare_canonical_basis, prepare_canonical_comparison, AspectValue, CanonicalComparisonOutcome,
    CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, CanonicalEquivalenceBasis,
    InternedString,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

use super::{account_parameter, installed_query};
use crate::application_query::parameter_canonical_basis::prepare_parameter_basis;
use crate::application_query::{
    admit_application_query_parameters, readmit_application_query_parameters,
};

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
fn mutation_selectors_match_exact_admitted_parameters() {
    let query = installed_query();
    let source = admitted(&query, 7);
    let expected = |account| {
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), account)
            .unwrap()
    };
    assert!(source.matches_expected(expected(7)).unwrap());
    assert!(!source.matches_expected(expected(8)).unwrap());
    assert!(!source
        .matches_expected(ApplicationQueryParameterSet::<super::ActivityQuery>::new())
        .unwrap());
    assert_eq!(source
        .matches_expected(expected(7).bind(account_parameter(), 7).unwrap())
        .unwrap_err().kind(),
        crate::application_query::WorthQueryApplicationQueryParameterDenialKind::CanonicalEntryBudgetExceeded);
}

#[test]
fn producer_parameter_comparison_admits_before_canonical_copy() {
    let query = installed_query();
    let source = admitted(&query, 7);
    let expected = |account| {
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), account)
            .unwrap()
    };
    let mut charges = Vec::new();
    assert!(source
        .matches_expected_with_preflight(expected(7), |work, bytes| {
            charges.push((work, bytes));
            Ok::<_, ()>(())
        })
        .unwrap()
        .unwrap());
    assert!(charges.iter().any(|(work, _)| *work > 0));
    assert!(charges.iter().any(|(_, bytes)| *bytes > 0));
    assert!(source
        .matches_expected_with_preflight(expected(8), |_, _| Ok::<_, ()>(()))
        .unwrap()
        .is_ok_and(|matches| !matches));
    assert!(source
        .matches_expected_with_preflight(expected(7), |_, _| Err::<(), _>(()))
        .is_err());
}

#[test]
fn retained_parameter_values_receive_a_fresh_installed_canonical_basis() {
    let query = installed_query();
    let retained = admitted(&query, 7);
    let readmitted = readmit_application_query_parameters(&query, &retained).unwrap();
    let independent = admitted(&query, 7);

    assert_eq!(readmitted.bindings(), retained.bindings());
    assert_eq!(readmitted.identity(), independent.identity());
    assert!(readmitted
        .canonical_basis()
        .is_equivalent_to(independent.canonical_basis()));
    assert_ne!(readmitted.identity(), admitted(&query, 8).identity());
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
