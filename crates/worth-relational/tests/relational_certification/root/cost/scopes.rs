use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use super::world::supply_chain::{
    commit_branch_batch, lower_supply_chain_production_delta, snapshot_for_supply_chain_identity,
    DeltaId, SupplyChainScale,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn phase5_cost_scope_excludes_prior_target_work_and_later_sibling_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    fork(&world.runtime, "before-capture");
    let observed_branch = world
        .runtime
        .branch_identity(&BranchId("before-capture".to_owned()))
        .expect("the owner issues the target identity after fork");
    let scope = RelationalMvccCostScope::capture(&world.runtime, vec![observed_branch]);
    fork(&world.runtime, "after-capture");
    let observation = world.runtime.observe_mvcc_cost(&scope).unwrap();
    assert_eq!(observation.sharing_cost_delta().shared_root_acquisitions, 0);
}

#[test]
fn phase5_selected_publication_tuple_is_stable_across_target_history_lengths() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut observed_costs = Vec::new();
    for history_len in [0, 8, 64] {
        let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::court());
        assert_oracle_matches(execution, &world, &expected);
        let branch_id = BranchId("medical-hold".to_owned());
        fork(&world.runtime, &branch_id.0);
        for sequence in 0..history_len {
            commit_branch_batch(
                &world.runtime,
                branch_id.clone(),
                WorkerIntentBatch::new(format!("history-{sequence}")),
            );
        }
        let selected = world
            .runtime
            .branch_identity(&branch_id)
            .expect("the branch remains owner-issued across history lengths");
        let scope = RelationalMvccCostScope::capture(&world.runtime, vec![selected]);
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &branch_id,
            &std::collections::BTreeSet::new(),
            DeltaId::HoldMedicalCargo,
        )
        .expect("the actual prestate admits the Medical delta");
        commit_branch_batch(&world.runtime, branch_id, batch);
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
fn phase5_selected_publication_tuple_ignores_unrelated_population_reads_and_validation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut observed_costs = Vec::new();
    for unrelated_branch_count in [2, 8, 64] {
        let (mut world, expected) =
            certified_supply_chain_world(execution, SupplyChainScale::court());
        assert_oracle_matches(execution, &world, &expected);
        let selected_branch = BranchId("medical-hold".to_owned());
        fork(&world.runtime, &selected_branch.0);
        let selected = world.runtime.branch_identity(&selected_branch).unwrap();
        let unrelated_branches = fork_unrelated_branches(&mut world, unrelated_branch_count);
        let unrelated_scope =
            RelationalMvccCostScope::capture(&world.runtime, vec![selected.clone()]);
        let unrelated_work_scope = RelationalMvccCostScope::capture(
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

        let publication_scope = RelationalMvccCostScope::capture(&world.runtime, vec![selected]);
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &selected_branch,
            &std::collections::BTreeSet::new(),
            DeltaId::HoldMedicalCargo,
        )
        .expect("the selected branch validates after unrelated work");
        commit_branch_batch(&world.runtime, selected_branch, batch);
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

fn fork_unrelated_branches(
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    branch_count: usize,
) -> Vec<(BranchId, RelationalBranchIdentity)> {
    (0..branch_count)
        .map(|ordinal| {
            let branch_name = match ordinal {
                0 => "storm".to_owned(),
                1 => "maintenance".to_owned(),
                _ => format!("unrelated-{ordinal}"),
            };
            let branch_id = BranchId(branch_name);
            fork(&world.runtime, &branch_id.0);
            let identity = world.runtime.branch_identity(&branch_id).unwrap();
            (branch_id, identity)
        })
        .collect()
}

fn exercise_unrelated_branch_reads_and_validation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    branches: &[(BranchId, RelationalBranchIdentity)],
) {
    for (ordinal, (branch_id, identity)) in branches.iter().enumerate() {
        let validation_delta = match ordinal {
            0 => Some(DeltaId::StormRerouteAurora),
            1 => Some(DeltaId::MaintainAtlasBerth),
            _ => None,
        };
        let snapshot = snapshot_for_supply_chain_identity(&world.runtime, identity);
        let inspection = world
            .runtime
            .read_truth()
            .inspect_snapshot(&snapshot)
            .expect("unrelated branch read stays branch-qualified");
        assert_eq!(&inspection.branch_id, branch_id);
        if let Some(delta) = validation_delta {
            let batch = lower_supply_chain_production_delta(
                execution,
                &world.runtime,
                &world.program,
                &world.handles,
                branch_id,
                &std::collections::BTreeSet::new(),
                delta,
            )
            .expect("named unrelated branch validates its own named delta");
            commit_branch_batch(&world.runtime, branch_id.clone(), batch);
        }
    }
}

fn fork(runtime: &worth_relational::facade::runtime::RelationalRuntime, name: &str) {
    let (_, source_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(BranchId(name.to_owned()), source_basis)
        .unwrap();
}
