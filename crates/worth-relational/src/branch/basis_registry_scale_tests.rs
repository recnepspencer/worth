use crate::history::data::BranchId;
fn prove_registry_lookup_work_is_population_independent(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch_population: usize,
) {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity(execution, &runtime, "basis-scale-root");
    populate_branches(execution, &runtime, branch_population);
    let (target, retained) = retain_one_basis_per_branch(&runtime, branch_population);
    let after = prove_single_readmission_is_fixed_work(&runtime, &target, branch_population);
    drop(retained);
    assert_exact_registry_cleanup(&runtime, after, branch_population);
}

fn populate_branches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    branch_population: usize,
) {
    for index in 1..branch_population {
        runtime
            .history_authority()
            .fork_branch_from(
                execution,
                BranchId(format!("basis-scale-{index}")),
                &BranchId("main".to_owned()),
            )
            .unwrap();
    }
}

fn retain_one_basis_per_branch(
    runtime: &crate::runtime::RelationalRuntime,
    branch_population: usize,
) -> (
    super::RelationalBranchBasisDescriptor,
    Vec<super::AdmittedRelationalBranchBasis>,
) {
    let mut retained = Vec::with_capacity(branch_population);
    let mut target = None;
    for index in 0..branch_population {
        let branch_id = if index == 0 {
            BranchId("main".to_owned())
        } else {
            BranchId(format!("basis-scale-{index}"))
        };
        let identity = runtime.branch_identity(&branch_id).unwrap();
        let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
        target.get_or_insert(descriptor);
        retained.push(basis);
    }
    (target.unwrap(), retained)
}

fn prove_single_readmission_is_fixed_work(
    runtime: &crate::runtime::RelationalRuntime,
    target: &super::RelationalBranchBasisDescriptor,
    branch_population: usize,
) -> super::RelationalBranchBasisCostCounters {
    let before = runtime.branch_basis_cost_counters();
    let population_scans_before = runtime
        .phase4_reference_cost_counters()
        .branch_population_scans;
    let readmitted = runtime.readmit_branch_basis(target).unwrap();
    let after = runtime.branch_basis_cost_counters();
    let population_scans_after = runtime
        .phase4_reference_cost_counters()
        .branch_population_scans;

    assert_eq!(
        before.retained_basis_registry_entries,
        branch_population as u64
    );
    assert_eq!(
        after.retained_basis_registry_entries,
        branch_population as u64
    );
    assert_eq!(
        after.retained_basis_registry_key_lookups,
        before.retained_basis_registry_key_lookups + 1
    );
    assert_eq!(
        after.retained_basis_registry_mutations,
        before.retained_basis_registry_mutations
    );
    assert_eq!(population_scans_after, population_scans_before);

    drop(readmitted);
    after
}

fn assert_exact_registry_cleanup(
    runtime: &crate::runtime::RelationalRuntime,
    after_readmission: super::RelationalBranchBasisCostCounters,
    branch_population: usize,
) {
    let cleaned = runtime.branch_basis_cost_counters();
    assert_eq!(cleaned.retained_basis_registry_entries, 0);
    assert_eq!(
        cleaned.retained_basis_registry_key_lookups,
        after_readmission.retained_basis_registry_key_lookups + branch_population as u64
    );
    assert_eq!(
        cleaned.retained_basis_registry_mutations,
        after_readmission.retained_basis_registry_mutations + branch_population as u64
    );
}

#[test]
fn retained_basis_registry_cost_is_fixed_at_one_sixty_four_and_four_thousand_ninety_six_branches() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for branch_population in [1, 64, 4_096] {
        prove_registry_lookup_work_is_population_independent(execution, branch_population);
    }
}

#[test]
fn ordinary_publication_does_not_scan_branch_or_branch_head_populations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for branch_population in [64, 4_096] {
        let runtime = crate::tests::support::runtime_with_test_schema();
        crate::tests::support::create_entity(execution, &runtime, "publication-scan-root");
        populate_branches(execution, &runtime, branch_population);
        let branch_scans_before = runtime
            .phase4_reference_cost_counters()
            .branch_population_scans;
        let visibility_before = runtime.visibility.visibility_cache_cost_counters();

        crate::tests::support::create_entity(
            execution,
            &runtime,
            &format!("publication-scan-{branch_population}"),
        );

        let branch_scans_after = runtime
            .phase4_reference_cost_counters()
            .branch_population_scans;
        let visibility_after = runtime.visibility.visibility_cache_cost_counters();
        assert_eq!(branch_scans_after, branch_scans_before);
        assert_eq!(
            visibility_after.branch_head_population_scans,
            visibility_before.branch_head_population_scans
        );
    }
}
