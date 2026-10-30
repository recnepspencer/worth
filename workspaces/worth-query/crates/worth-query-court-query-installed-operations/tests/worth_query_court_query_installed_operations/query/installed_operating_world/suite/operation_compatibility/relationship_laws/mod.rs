pub(crate) use worth_query_testkit_query::facade::query::suite::operation_compatibility::relationship_laws::*;

#[test]
fn installation_sameness_does_not_claim_basis_compatibility() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = no_primary_read_runtime()
        .workspace(
            request_execution,
            "compatibility-distinct-relationship-laws",
        )
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let current = bind_current(&workspace, &installed);
    let branch = bind_branch(&workspace, &installed);

    current.same_installation_with(&branch).unwrap();
    assert_eq!(
        current.compatible_basis_with(&branch).unwrap_err().kind(),
        domain::WorthQueryCompatibilityDenialKind::BasisMismatched
    );
}

#[test]
fn every_relationship_preserves_query_bound_conditional_drift() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = node(
        "compatibility-node",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let (mut owner, donor) = conditional_controlled_workspace_with_donor(
        request_execution,
        ConditionalDonorWorkspaceScenario {
            owner: ConditionalWorkspacePlacement {
                name: "compatibility-relationship-drift-owner",
                partition: "geometry-signal",
            },
            donor: ConditionalWorkspacePlacement {
                name: "compatibility-relationship-drift-donor",
                partition: "geometry-drifted-signal",
            },
            node: declaration.clone(),
            donor_compute: DirectConditionalCompute,
        },
    )
    .unwrap();
    let prior_domain = owner.domain(GeometryDomain).unwrap();
    let subject = bind(&owner, &prior_domain);

    owner
        .replace_conditional_lowerings_from::<GeometryDomain, ReadVertex, ReadFamily>(&donor)
        .unwrap();
    let candidate = bind(&owner, &prior_domain);
    assert_affinity_drift(subject.same_installation_with(&candidate).unwrap_err());
    assert_affinity_drift(subject.replacement_with(&candidate).unwrap_err());
    assert_continuity_drift(subject.compatible_basis_with(&candidate).unwrap_err());
    assert_affinity_drift(subject.execution_sharing_with(&candidate).unwrap_err());

    let (mut rebound_owner, rebound_donor) = conditional_controlled_workspace_with_donor(
        request_execution,
        ConditionalDonorWorkspaceScenario {
            owner: ConditionalWorkspacePlacement {
                name: "compatibility-rebind-drift-owner",
                partition: "geometry-signal",
            },
            donor: ConditionalWorkspacePlacement {
                name: "compatibility-rebind-drift-donor",
                partition: "geometry-drifted-signal",
            },
            node: declaration,
            donor_compute: DirectConditionalCompute,
        },
    )
    .unwrap();
    let rebound_prior_domain = rebound_owner.domain(GeometryDomain).unwrap();
    let rebound_subject = bind(&rebound_owner, &rebound_prior_domain);
    rebound_owner
        .advance_domain_installation_generation()
        .unwrap();
    let rebound = rebound_owner
        .rebind_domain(rebound_prior_domain.rebind_request())
        .unwrap();
    rebound_owner
        .replace_conditional_lowerings_from::<GeometryDomain, ReadVertex, ReadFamily>(
            &rebound_donor,
        )
        .unwrap();
    let rebound_candidate = bind(&rebound_owner, rebound.handle());
    assert_continuity_drift(
        rebound_subject
            .rebind_with(&rebound_candidate, rebound.receipt().clone())
            .unwrap_err(),
    );
}

#[test]
fn changed_required_domain_authority_requires_its_owner_rebind_receipt() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut controlled = required_domain_runtime(true)
        .controlled_workspace(request_execution, "compatibility-required-domain-rebind")
        .unwrap();
    let prior_geometry = controlled.domain(GeometryDomain).unwrap();
    let prior_auxiliary = controlled.domain(AuxiliaryDomain).unwrap();
    let subject = controlled
        .observe_operating_world(controlled.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&prior_geometry, ReadVertex)
        .unwrap();

    controlled.advance_domain_installation_generation().unwrap();
    let geometry = controlled
        .rebind_domain(prior_geometry.rebind_request())
        .unwrap();
    let auxiliary = controlled
        .rebind_domain(prior_auxiliary.rebind_request())
        .unwrap();
    let candidate = controlled
        .observe_operating_world(controlled.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(geometry.handle(), ReadVertex)
        .unwrap();

    let denial = subject
        .rebind_with(&candidate, geometry.receipt().clone())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryCompatibilityDenialKind::RequiredDomainAuthority
    );
    assert_eq!(
        denial.counters().required_domain_rebind_receipts_inspected,
        0
    );

    let unrelated = subject
        .rebind_with_required_domain_receipts(
            &candidate,
            geometry.receipt().clone(),
            vec![geometry.receipt().clone()],
        )
        .unwrap_err();
    assert_eq!(
        unrelated.kind(),
        domain::WorthQueryCompatibilityDenialKind::RequiredDomainAuthority
    );
    assert!(
        unrelated
            .counters()
            .required_domain_rebind_receipts_inspected
            > 0
    );

    subject
        .rebind_with_required_domain_receipts(
            &candidate,
            geometry.receipt().clone(),
            vec![auxiliary.receipt().clone()],
        )
        .unwrap();
}
