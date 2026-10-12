use super::*;

#[test]
fn handle_bound_contribution_matches_internal_oracle_and_retains_installed_witnesses() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let contributions = handle.contributions(&runtime).unwrap();
    let declaration = contribution_declaration("installed-authority");
    let target = contributions.intent_target(&declaration).unwrap();
    let materialized = contributions
        .for_intent_target(target.clone())
        .unwrap()
        .advises("accepts")
        .because("the installed domain accepts this declaration")
        .materialize()
        .unwrap();
    let requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "WORTH.tests.installed-domain.accepts",
        "the installed domain accepts this declaration",
    )
    .bind_to_installed_target(target.clone());
    let worth_proof::TransitionOutcome::Success(eligible) =
        evaluate_requested_domain_capability_contribution(requested)
    else {
        panic!("installed contribution oracle must be eligible")
    };
    let worth_proof::TransitionOutcome::Success(admitted) =
        admit_eligible_domain_capability_contribution(eligible)
    else {
        panic!("installed contribution oracle must be admitted")
    };
    let worth_proof::TransitionOutcome::Success(ready) =
        prepare_admitted_domain_capability_contribution_for_materialization(admitted, target)
    else {
        panic!("installed contribution oracle must be ready")
    };
    let oracle = materialize_canonical_admission_artifact(ready);
    assert_eq!(
        materialized.materialization_identity(),
        oracle.materialization_identity()
    );
    let authority = materialized.installed_authority().unwrap();
    assert_eq!(
        authority.authority_identity(),
        handle.authority().authority_identity()
    );
    assert_eq!(authority.package_identity(), handle.package_identity());
    assert_eq!(
        materialized.installed_world_identity(),
        Some(handle.authority().world_identity())
    );
}

#[test]
fn contribution_authority_denies_foreign_targets_and_uninstalled_categories_first() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = installed_runtime(request_execution);
    let right = installed_runtime(request_execution);
    let left_handle = left.domain(InstalledDomain).unwrap();
    let right_handle = right.domain(InstalledDomain).unwrap();
    let declaration = contribution_declaration("foreign-target");
    let foreign_target = left_handle
        .contributions(&left)
        .unwrap()
        .intent_target(&declaration)
        .unwrap();
    let denial = right_handle
        .contributions(&right)
        .unwrap()
        .for_intent_target(foreign_target)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryDomainHandleDenialKind::ForeignRuntime
    );
    let policy_denial = left_handle
        .contributions(&left)
        .unwrap()
        .for_intent(&declaration)
        .unwrap()
        .supports_capability("")
        .because("authority is checked before semantic content")
        .try_materialize();
    assert_eq!(
        policy_denial.denial().unwrap().kind(),
        WorthQueryDomainCapabilityProgressionDenialKind::ContributionCategoryNotInstalled
    );
}

#[test]
fn foreign_handle_and_stale_generation_deny_before_contribution_successors_are_issued() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let owner = installed_runtime(request_execution);
    let foreign = installed_runtime(request_execution);
    let owner_handle = owner.domain(InstalledDomain).unwrap();
    let foreign_denial = owner_handle.contributions(&foreign).unwrap_err();
    assert_eq!(
        foreign_denial.kind(),
        WorthQueryDomainHandleDenialKind::ForeignRuntime
    );

    let mut current = installed_runtime(request_execution);
    let stale_handle = current.domain(InstalledDomain).unwrap();
    let stale_surface = stale_handle.contributions(&current).unwrap();
    let stale_generation = stale_handle.installation_generation();
    current
        .replace_domain_installation_with_successor_generation()
        .unwrap();
    let current_handle = current.domain(InstalledDomain).unwrap();
    assert!(current_handle.installation_generation() > stale_generation);
    let stale_denial = stale_handle.contributions(&current).unwrap_err();
    assert_eq!(
        stale_denial.kind(),
        WorthQueryDomainHandleDenialKind::StaleInstallationGeneration
    );
    let stale_denial = stale_surface
        .for_intent(&contribution_declaration("stale-surface"))
        .unwrap_err();
    assert_eq!(
        stale_denial.kind(),
        WorthQueryDomainHandleDenialKind::StaleInstallationGeneration
    );
    current_handle
        .contributions(&current)
        .expect("the successor-generation handle must remain usable");
}

#[test]
fn generation_turnover_denies_each_query_owned_contribution_transition() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut admission_runtime = installed_runtime(request_execution);
    let admission_handle = admission_runtime.domain(InstalledDomain).unwrap();
    let admission_target = admission_handle
        .contributions(&admission_runtime)
        .unwrap()
        .intent_target(&contribution_declaration("stale-before-admission"))
        .unwrap();
    let admission_requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "WORTH.tests.installed-domain.stale-before-admission",
        "generation turnover must invalidate an eligible contribution",
    )
    .bind_to_installed_target(admission_target);
    let worth_proof::TransitionOutcome::Success(eligible) =
        evaluate_requested_domain_capability_contribution(admission_requested)
    else {
        panic!("fixture must become eligible before generation turnover")
    };
    admission_runtime
        .replace_domain_installation_with_successor_generation()
        .unwrap();
    let worth_proof::TransitionOutcome::Denied(admission_denial) =
        admit_eligible_domain_capability_contribution(eligible)
    else {
        panic!("stale eligible contribution must not produce an admitted successor")
    };
    assert_eq!(
        admission_denial.kind(),
        WorthQueryDomainCapabilityProgressionDenialKind::StaleInstallationGeneration
    );

    let mut preparation_runtime = installed_runtime(request_execution);
    let preparation_handle = preparation_runtime.domain(InstalledDomain).unwrap();
    let preparation_target = preparation_handle
        .contributions(&preparation_runtime)
        .unwrap()
        .intent_target(&contribution_declaration("stale-before-preparation"))
        .unwrap();
    let preparation_requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "WORTH.tests.installed-domain.stale-before-preparation",
        "generation turnover must invalidate an admitted contribution",
    )
    .bind_to_installed_target(preparation_target.clone());
    let worth_proof::TransitionOutcome::Success(eligible) =
        evaluate_requested_domain_capability_contribution(preparation_requested)
    else {
        panic!("fixture must become eligible before admission")
    };
    let worth_proof::TransitionOutcome::Success(admitted) =
        admit_eligible_domain_capability_contribution(eligible)
    else {
        panic!("fixture must become admitted before generation turnover")
    };
    preparation_runtime
        .replace_domain_installation_with_successor_generation()
        .unwrap();
    let worth_proof::TransitionOutcome::Denied(preparation_denial) =
        prepare_admitted_domain_capability_contribution_for_materialization(
            admitted,
            preparation_target,
        )
    else {
        panic!("stale admitted contribution must not become materialization-ready")
    };
    assert_eq!(
        preparation_denial.kind(),
        WorthQueryDomainCapabilityProgressionDenialKind::StaleInstallationGeneration
    );
}

#[test]
fn contribution_target_mismatch_requires_rebind_without_a_ready_successor() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = installed_runtime(request_execution);
    let handle = runtime.domain(InstalledDomain).unwrap();
    let contributions = handle.contributions(&runtime).unwrap();
    let bound_target = contributions
        .intent_target(&contribution_declaration("bound"))
        .unwrap();
    let current_target = contributions
        .intent_target(&contribution_declaration("current"))
        .unwrap();
    let requested = WorthQueryAdmissionContributionAuthoring::advisory(
        "WORTH.tests.installed-domain.rebind",
        "target identity is exact",
    )
    .bind_to_installed_target(bound_target);
    let worth_proof::TransitionOutcome::Success(eligible) =
        evaluate_requested_domain_capability_contribution(requested)
    else {
        panic!("fixture must be eligible")
    };
    let worth_proof::TransitionOutcome::Success(admitted) =
        admit_eligible_domain_capability_contribution(eligible)
    else {
        panic!("fixture must be admitted")
    };
    assert!(matches!(
        prepare_admitted_domain_capability_contribution_for_materialization(
            admitted,
            current_target
        ),
        worth_proof::TransitionOutcome::RebindRequired(_)
    ));
}

fn contribution_declaration(name: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        "installed-domain-test",
        "1.0",
        "installed-domain-test.v1",
        WorthQueryIntentInput::null(),
    )
}
