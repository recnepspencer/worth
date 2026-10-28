#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::root_branch_isolation::*;

#[test]
fn phase5_named_supply_chain_deltas_keep_sibling_roots_and_work_independent() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    let scenarios = [
        ("storm", BranchLabel::Storm, DeltaId::StormRerouteAurora),
        (
            "maintenance",
            BranchLabel::Maintenance,
            DeltaId::MaintainAtlasBerth,
        ),
        (
            "medical-hold",
            BranchLabel::MedicalHold,
            DeltaId::HoldMedicalCargo,
        ),
    ];
    for (branch, _, _) in scenarios {
        fork_from_main(execution, &world.runtime, branch);
    }
    let main_before = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main state");

    for (branch, label, delta) in scenarios {
        let selected = branch_identity(&world.runtime, branch);
        let unrelated_scopes = scenarios
            .iter()
            .filter(|(candidate, _, _)| *candidate != branch)
            .map(|(candidate, _, _)| branch_identity(&world.runtime, candidate))
            .map(|identity| RelationalMvccCostScope::capture(&world.runtime, vec![identity]))
            .collect::<Vec<_>>();
        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId(branch.to_owned()),
            &BTreeSet::new(),
            delta,
        )
        .expect("the selected branch's actual pre-state admits its named delta");
        commit_branch_batch(
            execution,
            &world.runtime,
            BranchId(branch.to_owned()),
            batch,
        );

        let observed = observe_branch(execution, &mut world, &selected, branch);
        compare(
            &expected_supply_chain_branch(&world.program, label, Some(delta)),
            &observed,
        )
        .expect("production branch state matches the independently authored oracle");
        for scope in &unrelated_scopes {
            let cost = world.runtime.observe_mvcc_cost(scope).unwrap();
            assert_eq!(
                cost.sharing_cost_delta(),
                RelationalBranchSharingCostCounters::default(),
                "a selected-branch write must perform zero storage work on each sibling"
            );
        }
    }

    let identities = [
        world.runtime.main_branch_identity(),
        branch_identity(&world.runtime, "storm"),
        branch_identity(&world.runtime, "maintenance"),
        branch_identity(&world.runtime, "medical-hold"),
    ];
    let sharing = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(sharing.unique_root_count(), 4);
    for (branch, label, delta) in scenarios {
        let identity = branch_identity(&world.runtime, branch);
        let observed = observe_branch(execution, &mut world, &identity, branch);
        compare(
            &expected_supply_chain_branch(&world.program, label, Some(delta)),
            &observed,
        )
        .expect("every sibling remains semantically isolated after all writes");
    }
    let main_identity = world.runtime.main_branch_identity();
    let main_observed = observe_branch(execution, &mut world, &main_identity, "main");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Operating, None),
        &main_observed,
    )
    .expect("main semantic truth remains unchanged after all sibling writes");
    let main_after = world
        .runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .expect("main state after sibling writes");
    assert_eq!(main_after, main_before);
}
