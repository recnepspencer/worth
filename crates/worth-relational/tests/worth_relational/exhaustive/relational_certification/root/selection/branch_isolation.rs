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

#[test]
fn main_advances_before_child_read_keeps_child_snapshot_on_fork_root() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork_from_main(execution, &world.runtime, "storm");
    let child = branch_identity(&world.runtime, "storm");
    let child_root_before_main_advance = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&child))
        .expect("child root is owner-inspectable")
        .root_ids()[0];

    advance_main_aurora_status(execution, &mut world);

    let main = world.runtime.main_branch_identity();
    let main_root_after_advance = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main))
        .expect("advanced main root is owner-inspectable")
        .root_ids()[0];
    assert_ne!(
        main_root_after_advance, child_root_before_main_advance,
        "the adversarial world must contain distinct R1 and retained child R0 roots"
    );

    let child_snapshot = snapshot_for_supply_chain_identity(&world.runtime, &child);
    let child_snapshot_inspection = world
        .runtime
        .read_truth()
        .inspect_snapshot(execution, &child_snapshot)
        .expect("snapshot inspection keeps the original execution request")
        .expect("child snapshot remains inspectable through the production read path");
    assert_eq!(
        child_snapshot_inspection.branch_id,
        BranchId("storm".to_owned())
    );
    assert_eq!(
        child_snapshot_inspection.root_id,
        Some(child_root_before_main_advance),
        "child read must retain R0 and never select main's newer R1"
    );
    let main_snapshot = snapshot_for_supply_chain_identity(&world.runtime, &main);
    assert_child_explicit_query_uses_snapshot_root(
        execution,
        &world,
        &child_snapshot,
        &main_snapshot,
    );
    let observed_child = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(child_snapshot.clone()),
        &world.runtime,
        &child_snapshot,
    )
    .expect("child remains semantically observable after main advances");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &observed_child,
    )
    .expect("unchanged child truth remains exactly its fork baseline");
}
