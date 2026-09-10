pub(super) mod fixture;

use worth_foundational::facade::{CanonicalBasisLocus, InternedString};
use worth_query::facade::{domain, foundation};

use super::super::installed_operation_fixture::{
    configured_runtime, support_dimension_workspace, GeometryDomain, ReadFamily, ReadVertex,
};
use fixture::{bind_branch, bind_current, no_primary_read_runtime};
type BoundVertex = domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;
const PORTABLE_OPERATION_COMPARISON_DIMENSIONS: usize = 45;

#[test]
fn all_five_relationship_oracles_are_stable_across_index_rebuild() {
    let mut controlled = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .controlled_workspace("compatibility-index-oracle")
        .unwrap();
    let prior_domain = controlled.domain(GeometryDomain).unwrap();
    let subject = bind_read_vertex(&controlled, &prior_domain);
    let candidate_before = bind_read_vertex(&controlled, &prior_domain);
    let before = current_relationship_counters(&subject, &candidate_before);

    assert!(controlled
        .verify_domain_execution_index_rebuild()
        .is_equivalent());
    assert!(controlled
        .rebuild_conditional_execution_index()
        .exact_index_parity());
    let candidate_after = bind_read_vertex(&controlled, &prior_domain);
    let after = current_relationship_counters(&subject, &candidate_after);
    assert_eq!(after, before);
    assert_current_success_costs(before);

    controlled.advance_domain_installation_generation().unwrap();
    let (current_domain, receipt) = controlled
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let rebound_before = bind_read_vertex(&controlled, &current_domain);
    let rebind_before = subject
        .rebind_with(&rebound_before, receipt.clone())
        .unwrap()
        .counters();

    assert!(controlled
        .verify_domain_execution_index_rebuild()
        .is_equivalent());
    assert!(controlled
        .rebuild_conditional_execution_index()
        .exact_index_parity());
    let rebound_after = bind_read_vertex(&controlled, &current_domain);
    let rebind_after = subject
        .rebind_with(&rebound_after, receipt)
        .unwrap()
        .counters();
    assert_eq!(rebind_after, rebind_before);
    assert_eq!(
        rebind_after.portable_contract_comparisons,
        PORTABLE_OPERATION_COMPARISON_DIMENSIONS
    );
    assert_eq!(rebind_after.canonical_comparisons, 5);
    assert_eq!(rebind_after.retained_authority_checks, 8);
    assert_eq!(rebind_after.portable_conditional_nodes_submitted, 0);
    assert_eq!(rebind_after.conditional_lowerings_compared, 0);
    assert_zero_forbidden_work(rebind_after);
}

#[test]
fn same_runtime_wrong_basis_and_stale_lifecycle_deny_through_compatibility() {
    let mut controlled = no_primary_read_runtime()
        .controlled_workspace("compatibility-basis-lifecycle")
        .unwrap();
    let prior_domain = controlled.domain(GeometryDomain).unwrap();
    let current = bind_current(&controlled, &prior_domain);
    let branch = bind_branch(&controlled, &prior_domain);

    let basis_denial = current.compatible_basis_with(&branch).unwrap_err();
    assert_eq!(
        basis_denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::BasisMismatched
    );
    assert!(matches!(
        basis_denial
            .canonical_mismatch()
            .and_then(|mismatch| mismatch.left_locus()),
        Some(CanonicalBasisLocus::Named(InternedString::Raw(locus))) if locus == "authority"
    ));
    assert_eq!(basis_denial.counters().canonical_comparisons, 5);
    assert_eq!(basis_denial.counters().retained_authority_checks, 10);
    assert_zero_forbidden_work(basis_denial.counters());

    controlled.advance_domain_installation_generation().unwrap();
    let stale_denial = current.same_installation_with(&branch).unwrap_err();
    assert_eq!(
        stale_denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::InstallationFreshness
    );
    assert_eq!(stale_denial.counters().canonical_comparisons, 0);
    assert_eq!(stale_denial.counters().retained_authority_checks, 2);
    assert_zero_forbidden_work(stale_denial.counters());
}

#[test]
fn matching_reporting_material_cannot_cross_a_foreign_runtime() {
    let owner = no_primary_read_runtime()
        .workspace("compatibility-collision")
        .unwrap();
    let foreign = no_primary_read_runtime()
        .workspace("compatibility-collision")
        .unwrap();
    let owner_bound = bind_current(&owner, &owner.domain(GeometryDomain).unwrap());
    let foreign_bound = bind_current(&foreign, &foreign.domain(GeometryDomain).unwrap());

    assert_eq!(
        owner_bound.definition().canonical_identity(),
        foreign_bound.definition().canonical_identity()
    );
    assert_eq!(owner_bound.basis_identity(), foreign_bound.basis_identity());
    assert_ne!(
        owner_bound.binding_identity(),
        foreign_bound.binding_identity()
    );
    let denial = owner_bound
        .same_installation_with(&foreign_bound)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::RuntimeAuthority
    );
    assert_eq!(denial.counters().retained_authority_checks, 1);
    assert_zero_forbidden_work(denial.counters());
}

#[test]
fn stale_and_current_lookalikes_cannot_claim_same_installation() {
    let mut controlled = no_primary_read_runtime()
        .controlled_workspace("compatibility-stale-collision")
        .unwrap();
    let prior_domain = controlled.domain(GeometryDomain).unwrap();
    let stale = bind_current(&controlled, &prior_domain);

    controlled.advance_domain_installation_generation().unwrap();
    let (current_domain, _) = controlled
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let current = bind_current(&controlled, &current_domain);

    assert_eq!(
        stale.definition().canonical_identity(),
        current.definition().canonical_identity()
    );
    assert_eq!(stale.basis_identity(), current.basis_identity());
    let denial = stale.same_installation_with(&current).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::InstallationFreshness
    );
    assert_eq!(denial.counters().canonical_comparisons, 0);
    assert_eq!(denial.counters().retained_authority_checks, 2);
    assert_zero_forbidden_work(denial.counters());
}

#[test]
fn execution_sharing_stops_after_the_first_unsupported_profile() {
    let workspace = no_primary_read_runtime()
        .workspace("compatibility-sharing-short-circuit")
        .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let subject = bind_current(&workspace, &domain);
    let candidate = bind_current(&workspace, &domain);

    let denial = subject.execution_sharing_with(&candidate).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::RelationshipRule
    );
    assert_eq!(denial.counters().retained_authority_checks, 13);
    assert_zero_forbidden_work(denial.counters());
}

#[test]
fn installation_owner_mismatch_category_survives_the_query_boundary() {
    let plain = configured_runtime()
        .workspace("compatibility-owner-category-plain")
        .unwrap();
    let sharing = support_dimension_workspace(
        "compatibility-owner-category-sharing",
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .unwrap();
    let left = bind_read_vertex(&plain, &plain.domain(GeometryDomain).unwrap());
    let right = bind_read_vertex(&sharing, &sharing.domain(GeometryDomain).unwrap());

    let denial = left.compatible_basis_with(&right).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::PortableOperationContract
    );
    assert_eq!(
        denial.portable_operation_dimension(),
        Some(&domain::WorthQueryPortableOperationDimension::Support(
            domain::WorthQueryPortableOperationSupportDimension::Sharing
        ))
    );
    assert_eq!(
        denial.portable_operation_mismatch_category(),
        Some(domain::WorthQueryPortableOperationComparisonMismatchCategory::InstallationOwner)
    );
    assert!(denial.canonical_mismatch().is_none());
    assert_eq!(denial.counters().retained_authority_checks, 0);
    assert_zero_forbidden_work(denial.counters());
}

fn current_relationship_counters(
    subject: &BoundVertex,
    candidate: &BoundVertex,
) -> [domain::WorthQueryCompatibilityCounters; 4] {
    [
        subject
            .same_installation_with(candidate)
            .unwrap()
            .counters(),
        subject.replacement_with(candidate).unwrap().counters(),
        subject.compatible_basis_with(candidate).unwrap().counters(),
        subject
            .execution_sharing_with(candidate)
            .unwrap()
            .counters(),
    ]
}

fn assert_current_success_costs(counters: [domain::WorthQueryCompatibilityCounters; 4]) {
    assert_eq!(
        counters.map(|counters| counters.retained_authority_checks),
        [10, 12, 10, 17]
    );
    assert_eq!(counters[0].portable_contract_comparisons, 0);
    assert_eq!(counters[0].portable_variable_items_submitted, 0);
    assert_eq!(counters[0].canonical_comparisons, 0);
    for counters in &counters[1..] {
        assert_eq!(
            counters.portable_contract_comparisons,
            PORTABLE_OPERATION_COMPARISON_DIMENSIONS
        );
        assert_eq!(counters.canonical_comparisons, 5);
        assert!(counters.portable_variable_items_submitted > 0);
        assert_eq!(counters.portable_conditional_nodes_submitted, 0);
        assert_eq!(counters.conditional_lowerings_compared, 0);
        assert_zero_forbidden_work(*counters);
    }
}

fn assert_zero_forbidden_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.lower_runtime_contacts, 0);
    assert_eq!(counters.execution_calls, 0);
    assert_eq!(counters.maintenance_calls, 0);
}

fn bind_read_vertex(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, ReadVertex)
        .unwrap()
}
