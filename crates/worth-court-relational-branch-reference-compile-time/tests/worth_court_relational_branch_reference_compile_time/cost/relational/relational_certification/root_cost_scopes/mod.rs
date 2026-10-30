#[allow(unused_imports)] use crate::relational_certification::reference_context::root_cost_scopes::*;
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta};
pub(crate) use {worth_relational::facade::history::BranchId};

#[test]
fn selected_publication_tuple_is_stable_across_target_history_lengths() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut observed_costs = Vec::new();
    for history_len in [0, 8, 64] {
        let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
        assert_oracle_matches(execution, &world, &expected);
        let branch_id = BranchId("medical-hold".to_owned());
        fork(execution, &world.runtime, &branch_id.0);
        for sequence in 0..history_len {
            commit_branch_batch(
                execution,
                &world.runtime,
                branch_id.clone(),
                worth_relational::facade::transactions::WorkerIntentBatch::new(format!(
                    "history-{sequence}"
                )),
            );
        }
        let selected = world
            .runtime
            .branch_identity(&branch_id)
            .expect("the branch remains owner-issued across history lengths");
        let scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
            &world.runtime,
            vec![selected],
        );
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &branch_id,
            &std::collections::BTreeSet::new(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
        )
        .expect("the actual prestate admits the Medical delta");
        commit_branch_batch(execution, &world.runtime, branch_id, batch);
        let observation = world.runtime.observe_mvcc_cost(&scope).unwrap();
        let publication = observation.sharing_cost_delta();
        observed_costs.push((
            publication.publication_touched_region_count,
            publication.publication_persistent_index_path_nodes,
            publication.branch_population_scans,
            publication.copied_commit_envelopes,
            publication.snapshot_root_reads,
            publication.transaction_validation_attempts,
            publication.retained_history_head_lookups,
            publication.candidate_preparations,
            publication.publication_attempts,
        ));
        assert_eq!(observation.branch_population_scans(), 0);
        assert_eq!(
            observation.sharing_cost_delta().branch_population_scans,
            0,
            "synchronous global retention maintenance stays outside the selected-branch tuple"
        );
        assert_eq!(
            observation
                .sharing()
                .inspection_reconstructed_region_count(),
            observation.sharing().region_locators().len() as u64,
        );
    }
    assert_eq!(observed_costs, vec![(1, 33, 0, 0, 0, 1, 1, 1, 1); 3]);
}

#[test]
fn selected_publication_tuple_ignores_unrelated_population_reads_and_validation() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut observed_costs = Vec::new();
    for unrelated_branch_count in [2, 8, 64] {
        let (mut world, expected) =
            certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
        assert_oracle_matches(execution, &world, &expected);
        let selected_branch = BranchId("medical-hold".to_owned());
        fork(execution, &world.runtime, &selected_branch.0);
        let selected = world.runtime.branch_identity(&selected_branch).unwrap();
        let unrelated_branches =
            fork_unrelated_branches(execution, &mut world, unrelated_branch_count);
        let unrelated_scope =
            worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                &world.runtime,
                vec![selected.clone()],
            );
        let unrelated_work_scope =
            worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                &world.runtime,
                unrelated_branches
                    .iter()
                    .map(|(_, identity)| identity.clone())
                    .collect(),
            );

        exercise_unrelated_branch_reads_and_validation(execution, &mut world, &unrelated_branches);

        let unrelated_cost = world.runtime.observe_mvcc_cost(&unrelated_scope).unwrap();
        assert_eq!(
            unrelated_cost.sharing_cost_delta(),
            worth_relational::facade::inspection::RelationalBranchSharingCostCounters::default(),
            "unrelated branch population, reads, and lowering cannot charge selected-branch work"
        );
        assert_eq!(unrelated_cost.branch_cell_contacts(), 0);
        assert_eq!(
            unrelated_cost.branch_population_scans(),
            0,
            "ordinary sibling publications do not enter population-wide reconstruction"
        );
        let unrelated_work = world
            .runtime
            .observe_mvcc_cost(&unrelated_work_scope)
            .unwrap()
            .sharing_cost_delta();
        assert_eq!(
            unrelated_work.snapshot_root_reads, unrelated_branch_count as u64,
            "Phase-6 exact-observation lowering does not mint snapshot handles"
        );
        assert_eq!(unrelated_work.transaction_validation_attempts, 2);
        assert_eq!(unrelated_work.retained_history_head_lookups, 2);
        assert_eq!(unrelated_work.candidate_preparations, 2);
        assert_eq!(unrelated_work.publication_attempts, 2);

        let publication_scope =
            worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                &world.runtime,
                vec![selected],
            );
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &selected_branch,
            &std::collections::BTreeSet::new(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
        )
        .expect("the selected branch validates after unrelated work");
        commit_branch_batch(execution, &world.runtime, selected_branch, batch);
        let publication = world.runtime.observe_mvcc_cost(&publication_scope).unwrap();
        let delta = publication.sharing_cost_delta();
        observed_costs.push((
            delta.publication_touched_region_count,
            delta.publication_persistent_index_path_nodes,
            delta.branch_population_scans,
            delta.copied_commit_envelopes,
            delta.snapshot_root_reads,
            delta.transaction_validation_attempts,
            delta.retained_history_head_lookups,
            delta.candidate_preparations,
            delta.publication_attempts,
        ));
        assert_eq!(
            publication.branch_cell_contacts(),
            2,
            "admission and publication each contact only the selected branch reference cell"
        );
        assert_eq!(publication.branch_population_scans(), 0);
        assert_eq!(
            publication.sharing_cost_delta().branch_population_scans,
            0,
            "unrelated branch-head maintenance is reported separately from selected work"
        );
    }
    assert_eq!(observed_costs, vec![(1, 33, 0, 0, 0, 1, 1, 1, 1); 3]);
}
