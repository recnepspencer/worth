//! Generated components share emission; explicit inputs retain validator refusal.
use super::super::{DeclaredConditionalBinding, WorthQueryApplicationContractCatalog};
use super::TypeId;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryApplicationConditionalPackageContract as Contract;
use crate::domain_computation::primary_graph::tests::fixture::{
    IdentityExecutionSchema as Schema, PatchAccountDraftInput, PatchAccountDraftOperation,
    TouchAccountInput, TouchAccountOperation,
};
use domain::ApplicationSchema;
use worth_query_installation::facade as domain;
mod fixture;
type Catalog = WorthQueryApplicationContractCatalog<Schema>;

#[test]
fn generated_shared_domain_validates_distinct_bindings_and_keeps_runtime_entries() {
    let shared = fixture::definition("composition-shared", "composition-v1");
    let [first, second] = fixture::contracts(&shared, &shared);
    let expected = [
        first.operation_binding().clone(),
        second.operation_binding().clone(),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    // The third runtime declaration reuses the first portable binding/target,
    // but has its own identity, type and producer association.
    let catalog = fixture::catalog(vec![first.clone(), second, first]);
    let before = fixture::snapshot(&catalog);
    let validated = catalog
        .compose_package(fixture::package())
        .validate()
        .unwrap();
    assert_eq!(validated.domain_operations(), &[shared.into_portable()]);
    assert_eq!(
        validated
            .conditional_application_operations()
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    assert_eq!(validated.conditional_application_operations().len(), 2);
    assert_eq!(
        fixture::snapshot(&catalog),
        before,
        "all target/owner/type/producer scopes survive"
    );
    assert_eq!(
        catalog.conditionals.len(),
        3,
        "portable reuse must not erase runtime registrations"
    );
    assert_eq!(before[0].2.target(), before[2].2.target());
    assert_ne!(before[0].3, before[2].3);
    assert_ne!(before[0].4, before[2].4);
}

#[test]
fn generated_domain_and_application_binding_conflicts_reach_package_validation() {
    let original = fixture::definition("composition-shared", "composition-v1");
    let drifted = fixture::definition("composition-shared", "composition-v2");
    let domain_slot = original.identity().slot();
    let contracts = fixture::contracts(&original, &drifted);
    let catalog = fixture::catalog(contracts.into());
    let denial = catalog
        .compose_package(fixture::package())
        .validate()
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryPortablePackageValidationDenialKind::ConflictingDefinition
    );
    assert_eq!(
        denial.definition_kind(),
        Some(domain::WorthQueryPortableDefinitionKind::DomainOperation)
    );
    assert_eq!(denial.slot(), domain_slot);

    // Different full domains are lawful, but a single application-operation
    // key cannot choose between their different exact meanings.
    let other = fixture::definition("composition-other", "composition-v1");
    let first = fixture::contracts(&original, &original)[0].clone();
    let conflicting = fixture::contracts(&other, &other)[0].clone();
    let catalog = fixture::catalog(vec![first, conflicting]);
    let denial = catalog
        .compose_package(fixture::package())
        .validate()
        .unwrap_err();
    assert_eq!(denial.kind(), domain::WorthQueryPortablePackageValidationDenialKind::ConflictingConditionalApplicationOperation);
    assert_eq!(denial.slot(), TouchAccountOperation::reference().name());
}

#[test]
fn explicit_incoming_domain_and_binding_duplicates_remain_refusals() {
    let shared = fixture::definition("composition-shared", "composition-v1");
    let [first, second] = fixture::contracts(&shared, &shared);
    let portable = shared.into_portable();
    let binding = first.operation_binding().clone();
    let catalog = fixture::catalog(vec![first, second]);
    // One explicit component colliding with generated meaning, and duplicate
    // components already incoming, must both reach the ordinary validator.
    for copies in [1, 2] {
        let mut incoming = fixture::package();
        for _ in 0..copies {
            incoming = incoming.domain_operation(portable.clone());
        }
        let denial = catalog.compose_package(incoming).validate().unwrap_err();
        assert_eq!(
            denial.kind(),
            domain::WorthQueryPortablePackageValidationDenialKind::DuplicateDefinition
        );
        assert_eq!(
            denial.definition_kind(),
            Some(domain::WorthQueryPortableDefinitionKind::DomainOperation)
        );
        assert_eq!(denial.slot(), portable.identity().slot());

        let mut incoming = fixture::package();
        for _ in 0..copies {
            incoming = incoming.conditional_application_operation_erased(binding.clone());
        }
        let denial = catalog.compose_package(incoming).validate().unwrap_err();
        assert_eq!(denial.kind(), domain::WorthQueryPortablePackageValidationDenialKind::DuplicateConditionalApplicationOperation);
        assert_eq!(denial.slot(), binding.application_operation());
    }
}
