pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn fork_unrelated_branches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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
            fork(execution, &world.runtime, &branch_id.0);
            let identity = world.runtime.branch_identity(&branch_id).unwrap();
            (branch_id, identity)
        })
        .collect()
}

pub(crate) fn exercise_unrelated_branch_reads_and_validation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
    branches: &[(BranchId, RelationalBranchIdentity)],
) {
    for (ordinal, (branch_id, identity)) in branches.iter().enumerate() {
        let validation_delta = match ordinal {
            0 => Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora),
            1 => Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth),
            _ => None,
        };
        let snapshot = snapshot_for_supply_chain_identity(&world.runtime, identity);
        let inspection = world
            .runtime
            .read_truth()
            .inspect_snapshot(execution, &snapshot)
            .expect("snapshot inspection keeps the original execution request")
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
            commit_branch_batch(execution, &world.runtime, branch_id.clone(), batch);
        }
    }
}

pub(crate) fn fork(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    name: &str,
) {
    let (_, source_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(name.to_owned()), source_basis)
        .unwrap();
}