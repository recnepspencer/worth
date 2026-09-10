use super::*;

#[test]
fn rebind_reissues_current_authority_only_for_equivalent_package_meaning() {
    let prior_runtime = installed_runtime();
    let current_runtime = installed_runtime();
    let prior = prior_runtime.domain(InstalledDomain).unwrap();
    let rebound = current_runtime
        .rebind_domain(prior.rebind_request())
        .unwrap();
    assert_eq!(
        rebound.handle().package_identity(),
        prior.package_identity()
    );
    assert_ne!(
        rebound.receipt().prior_witness_identity(),
        rebound.receipt().current_witness_identity()
    );
    current_runtime
        .validate_installed_domain_handle(rebound.handle())
        .unwrap();
    let changed = changed_package_runtime();
    let denial = changed.rebind_domain(prior.rebind_request()).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryDomainRebindDenialKind::PackageMeaningChanged
    );
    assert_eq!(
        denial.next_action(),
        WorthQueryDomainRebindNextAction::ReconcilePackageMeaning
    );
    assert_ne!(
        denial.prior_package_identity(),
        denial.current_package_identity().unwrap()
    );
    assert_eq!(denial.counters().planning_attempts(), 0);
    assert_eq!(denial.counters().lower_runtime_attempts(), 0);
    assert_eq!(denial.counters().execution_attempts(), 0);
}

#[test]
fn rebind_to_runtime_without_the_domain_prescribes_installation() {
    let prior_runtime = installed_runtime();
    let prior = prior_runtime.domain(InstalledDomain).unwrap();
    let empty_runtime = complete_backend_from_parts_builder()
        .build_backend_from_parts()
        .build()
        .unwrap();
    let denial = empty_runtime
        .rebind_domain(prior.rebind_request())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryDomainRebindDenialKind::DomainNotInstalled
    );
    assert_eq!(
        denial.next_action(),
        WorthQueryDomainRebindNextAction::InstallDomainPackage
    );
}

#[test]
fn generation_turnover_rebinds_to_the_current_runtime_witness() {
    let mut runtime = installed_runtime();
    let prior = runtime.domain(InstalledDomain).unwrap();
    let prior_generation = prior.installation_generation();
    let source = runtime.domain_installation_registry.retain_portable_index();
    runtime
        .replace_domain_installation_with_successor_generation()
        .unwrap();

    let rebound = runtime.rebind_domain(prior.rebind_request()).unwrap();
    assert_eq!(
        rebound.handle().package_identity(),
        prior.package_identity()
    );
    assert!(rebound.handle().installation_generation() > prior_generation);
    assert!(std::sync::Arc::ptr_eq(
        &source,
        &runtime.domain_installation_registry.retain_portable_index(),
    ));
    assert!(std::ptr::eq(
        source.as_ref(),
        runtime.execution_runtime.installed_packages(),
    ));
    assert_ne!(
        rebound.receipt().prior_witness_identity(),
        rebound.receipt().current_witness_identity()
    );
    runtime
        .validate_installed_domain_handle(rebound.handle())
        .unwrap();
}
