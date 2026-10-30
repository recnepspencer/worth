pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn single_record_copy_cost(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    scale: SupplyChainScale,
) -> u64 {
    let (world, expected) = certified_supply_chain_world(execution, scale);
    assert_oracle_matches(execution, &world, &expected);
    let branch = BranchId("single-record-copy".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    world
        .runtime
        .fork_branch(execution, branch.clone(), source)
        .unwrap();
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![identity],
    );
    let batch = lower_cargo_footprint_batch(&world.handles, scale, 1);
    let committed = commit_branch_batch_with_result(execution, &world.runtime, branch, batch);
    assert_eq!(
        committed.publication_summary().unwrap().patch_record_count,
        1
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let copied = world
        .runtime
        .observe_mvcc_cost(&scope)
        .unwrap()
        .sharing_cost_delta()
        .copied_truth_bytes;
    assert!(copied > 0);
    copied
}

pub(crate) fn assert_flat_history_samples(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history_lengths: &[usize],
) {
    let mut samples = Vec::new();
    for &history_len in history_lengths {
        let (mut world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
        let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
        let branch = BranchId(format!("history-{history_len}"));
        let (_, source) = world
            .runtime
            .observe_fork_source(&BranchId("main".to_owned()))
            .unwrap();
        world
            .runtime
            .fork_branch(execution, branch.clone(), source)
            .unwrap();
        let first_batch = lower_cargo_footprint_batch(&world.handles, scale, 1);
        let first =
            commit_branch_batch_with_result(execution, &world.runtime, branch.clone(), first_batch);
        world
            .runtime
            .snapshots()
            .release_snapshot(&first.snapshot)
            .unwrap();
        let identity = world.runtime.branch_identity(&branch).unwrap();
        let (_, first_basis) = world.runtime.observe_branch(&identity).unwrap();
        let mut retained_bases = vec![world.runtime.retain_component_basis(&first_basis).unwrap()];
        for sequence in 1..history_len {
            let committed = commit_branch_batch_with_result(
                execution,
                &world.runtime,
                branch.clone(),
                worth_relational::facade::transactions::WorkerIntentBatch::new(format!(
                    "retained-history-{sequence}"
                )),
            );
            world
                .runtime
                .snapshots()
                .release_snapshot(&committed.snapshot)
                .unwrap();
            let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
            retained_bases.push(world.runtime.retain_component_basis(&basis).unwrap());
        }
        assert_eq!(retained_bases.len(), history_len);
        let scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
            &world.runtime,
            vec![identity],
        );
        let final_batch = lower_cargo_footprint_batch(&world.handles, scale, 64);
        let final_commit =
            commit_branch_batch_with_result(execution, &world.runtime, branch, final_batch);
        let retained = world.runtime.run_branch_root_reclamation_pass();
        assert!(retained.roots_still_retained() >= history_len as u64);
        let delta = world
            .runtime
            .observe_mvcc_cost(&scope)
            .unwrap()
            .sharing_cost_delta();
        samples.push((
            delta.publication_touched_region_count,
            delta.publication_persistent_index_path_nodes,
            delta.branch_population_scans,
            delta.copied_commit_envelopes,
            delta.transaction_validation_attempts,
            delta.retained_history_head_lookups,
            delta.candidate_preparations,
            delta.publication_attempts,
        ));
        for retained_basis in retained_bases {
            world
                .runtime
                .release_component_basis(retained_basis)
                .unwrap();
        }
        world
            .runtime
            .snapshots()
            .release_snapshot(&final_commit.snapshot)
            .unwrap();
    }
    assert!(samples.windows(2).all(|pair| pair[0] == pair[1]));
}
