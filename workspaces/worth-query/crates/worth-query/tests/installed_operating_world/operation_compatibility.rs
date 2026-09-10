mod conditional_boundary_matrix;
mod hostile_matrix;
mod relationship_laws;

use worth_query::facade::{domain, foundation};

use super::conditional_node_contract::node;
use super::installed_operation_fixture::{
    conditional_controlled_workspace, conditional_workspace, configured_runtime,
    support_dimension_workspace, workspace, GeometryDomain, ReadFamily, ReadVertex,
};

#[test]
fn five_relationships_are_distinct_pair_bound_decisions() {
    let workspace = support_dimension_workspace(
        "operation-compatibility",
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let subject = bind(&workspace, &installed);
    let candidate = bind(&workspace, &installed);

    subject.same_installation_with(&candidate).unwrap();
    subject.replacement_with(&candidate).unwrap();
    subject.compatible_basis_with(&candidate).unwrap();
    subject.execution_sharing_with(&candidate).unwrap();

    assert_eq!(
        subject.replacement_with(&subject).unwrap_err().kind(),
        domain::WorthQueryCompatibilityDenialKind::RelationshipRule
    );
    assert_eq!(
        subject.execution_sharing_with(&subject).unwrap_err().kind(),
        domain::WorthQueryCompatibilityDenialKind::RelationshipRule
    );
}

#[test]
fn foreign_runtime_is_denied_before_operational_work() {
    let owner = workspace("operation-compatibility-owner", false).unwrap();
    let foreign = workspace("operation-compatibility-foreign", false).unwrap();
    let owner_bound = bind(&owner, &owner.domain(GeometryDomain).unwrap());
    let foreign_bound = bind(&foreign, &foreign.domain(GeometryDomain).unwrap());

    let denial = owner_bound
        .same_installation_with(&foreign_bound)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::RuntimeAuthority
    );
    assert_eq!(denial.counters().lower_runtime_contacts, 0);
    assert_eq!(denial.counters().execution_calls, 0);
    assert_eq!(denial.counters().maintenance_calls, 0);
}

#[test]
fn a_wrong_basis_is_rejected_before_a_bound_capability_can_exist() {
    let workspace = workspace("operation-compatibility-basis", false).unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let branch = workspace
        .branches()
        .fork(workspace.current_world())
        .components(|components| components.reuse_exact_relational_basis().fork_signal())
        .create()
        .unwrap();
    let denial = match workspace
        .observe_operating_world(branch)
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
    {
        Ok(_) => panic!("wrong basis must not mint a bound capability"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::BasisExecutionUnsupported
    );
    assert_eq!(denial.counters().planning_steps, 0);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
}

#[test]
fn rebuilt_indexes_do_not_change_the_authority_oracle() {
    let mut workspace = workspace("operation-compatibility-rebuild", false).unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let before = bind(&workspace, &installed);
    assert!(workspace
        .verify_domain_execution_index_rebuild()
        .is_equivalent());
    let report = workspace.rebuild_conditional_execution_index();
    assert!(report.exact_index_parity());
    let after = bind(&workspace, &installed);

    before.same_installation_with(&after).unwrap();
    before.compatible_basis_with(&after).unwrap();
}

#[test]
fn rebind_requires_a_stale_subject_and_current_same_runtime_successor() {
    let mut controlled = configured_runtime()
        .controlled_workspace("operation-compatibility-rebind")
        .unwrap();
    let prior_domain = controlled.domain(GeometryDomain).unwrap();
    let subject = bind(&controlled, &prior_domain);
    let same_generation_receipt = controlled
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .receipt()
        .clone();

    assert_eq!(
        subject
            .rebind_with(&subject, same_generation_receipt.clone())
            .unwrap_err()
            .kind(),
        domain::WorthQueryCompatibilityDenialKind::InstallationFreshness
    );

    controlled.advance_domain_installation_generation().unwrap();
    let (current_domain, receipt) = controlled
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let candidate = bind(&controlled, &current_domain);
    assert_eq!(
        subject
            .rebind_with(&candidate, same_generation_receipt)
            .unwrap_err()
            .kind(),
        domain::WorthQueryCompatibilityDenialKind::DomainRebindAuthority
    );
    subject.rebind_with(&candidate, receipt.clone()).unwrap();

    let foreign = workspace("operation-compatibility-rebind-foreign", false).unwrap();
    let foreign_bound = bind(&foreign, &foreign.domain(GeometryDomain).unwrap());
    let denial = subject.rebind_with(&foreign_bound, receipt).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::RuntimeAuthority
    );
    assert_eq!(denial.counters().lower_runtime_contacts, 0);
}

#[test]
fn conditional_rebind_reinstalls_through_successor_owners() {
    let mut controlled = conditional_controlled_workspace(
        "operation-compatibility-conditional-rebind",
        node(
            "conditional-rebind-node",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let prior_domain = controlled.domain(GeometryDomain).unwrap();
    let subject = bind(&controlled, &prior_domain);

    controlled.advance_domain_installation_generation().unwrap();
    let (current_domain, receipt) = controlled
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let candidate = bind(&controlled, &current_domain);

    let stale_denial = subject.same_installation_with(&candidate).unwrap_err();
    assert_eq!(
        stale_denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::InstallationFreshness
    );
    assert_zero_conditional_owner_work(stale_denial.counters());

    let rebind = subject.rebind_with(&candidate, receipt).unwrap();
    assert_conditional_continuity_work(rebind.counters());

    let current_peer = bind(&controlled, &current_domain);
    let affinity = candidate.same_installation_with(&current_peer).unwrap();
    assert_conditional_affinity_work(affinity.counters());
}

#[test]
fn one_field_conditional_drift_returns_the_exact_owner_dimension() {
    let left = conditional_workspace(
        "operation-compatibility-conditional-left",
        node(
            "compatibility-node",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let right = conditional_workspace(
        "operation-compatibility-conditional-right",
        node(
            "compatibility-node",
            domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let left_bound = bind(&left, &left.domain(GeometryDomain).unwrap());
    let right_bound = bind(&right, &right.domain(GeometryDomain).unwrap());

    let denial = left_bound.compatible_basis_with(&right_bound).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::PortableConditionalMismatched
    );
    assert!(matches!(
        denial.portable_conditional_dimension(),
        Some(
            domain::WorthQueryOperationConditionalDimension::Declaration {
                dimension: domain::WorthQueryPortableConditionalDimension::DependencyComparator,
                ..
            }
        )
    ));
    assert!(denial.canonical_mismatch().is_some());
    assert_eq!(
        denial.portable_operation_mismatch_category(),
        Some(domain::WorthQueryPortableOperationComparisonMismatchCategory::Foundational)
    );
    assert_eq!(denial.counters().lower_runtime_contacts, 0);
}

fn bind(
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

fn assert_conditional_continuity_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 1);
    assert_eq!(counters.conditional_foundational_comparisons, 31);
    assert_eq!(counters.conditional_liveness_checks, 1);
    assert_eq!(counters.conditional_correspondences_inspected, 1);
    assert_eq!(counters.conditional_targets_inspected, 1);
    assert_eq!(counters.conditional_provider_roles_inspected, 7);
    assert_eq!(counters.conditional_signal_semantic_dimensions_inspected, 8);
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 0);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 0);
}

fn assert_conditional_affinity_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 1);
    assert_eq!(counters.conditional_foundational_comparisons, 31);
    assert_eq!(counters.conditional_liveness_checks, 2);
    assert_eq!(counters.conditional_correspondences_inspected, 2);
    assert_eq!(counters.conditional_targets_inspected, 2);
    assert_eq!(counters.conditional_provider_roles_inspected, 14);
    assert_eq!(
        counters.conditional_signal_semantic_dimensions_inspected,
        16
    );
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 7);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 1);
}

fn assert_zero_conditional_owner_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 0);
    assert_eq!(counters.conditional_foundational_comparisons, 0);
    assert_eq!(counters.conditional_liveness_checks, 0);
    assert_eq!(counters.conditional_correspondences_inspected, 0);
    assert_eq!(counters.conditional_targets_inspected, 0);
    assert_eq!(counters.conditional_provider_roles_inspected, 0);
    assert_eq!(counters.conditional_signal_semantic_dimensions_inspected, 0);
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 0);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 0);
}
