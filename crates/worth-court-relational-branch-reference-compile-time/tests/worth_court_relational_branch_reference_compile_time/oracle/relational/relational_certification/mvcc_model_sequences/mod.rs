#[allow(unused_imports)]
use crate::relational_certification::reference_context::mvcc_model_sequences::*;
pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalMvccCostScope};

#[test]
fn all_declared_supply_chain_deltas_match_oracle_and_isolate_cost() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);

    let execution_order = [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
    ];
    let scenarios = execution_order.map(|delta| (branch_id(delta), delta));
    for (branch, _) in &scenarios {
        fork_from_main(execution, &world.runtime, branch.clone());
    }
    let identities = all_identities(&world.runtime, &scenarios);
    let shared = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(shared.branch_count(), 9);
    assert_eq!(shared.unique_root_count(), 1);
    assert_eq!(shared.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(shared.copied_commit_envelopes(), 0);
    let rewire_before = world
        .runtime
        .branch_reference_state(&BranchId("rewire".to_owned()))
        .unwrap();
    let mut committed_deltas = Vec::new();

    for (branch, delta) in scenarios.iter().cloned() {
        let references_before = scenarios
            .iter()
            .map(|(candidate, _)| {
                world
                    .runtime
                    .branch_reference_state(candidate)
                    .expect("every sibling has an owner reference")
            })
            .collect::<Vec<_>>();
        let selected = world.runtime.branch_identity(&branch).unwrap();
        let selected_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
            &world.runtime,
            vec![selected.clone()],
        );
        let unrelated = identities
            .iter()
            .filter(|identity| *identity != &selected)
            .cloned()
            .collect::<Vec<_>>();
        let unrelated_scope =
            worth_relational::facade::inspection::RelationalMvccCostScope::capture(
                &world.runtime,
                unrelated,
            );

        let batch = lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &branch,
            &std::collections::BTreeSet::new(),
            delta,
        )
        .expect("the production lowerer accepts the selected branch's actual state");
        let committed = commit_supply_chain_delta(
            execution,
            &world.runtime,
            &world.program,
            branch.clone(),
            delta,
            batch,
        );
        committed_deltas.push(delta);
        for (ordinal, (candidate, _)) in scenarios.iter().enumerate() {
            let after = world.runtime.branch_reference_state(candidate).unwrap();
            if candidate == &branch {
                assert_ne!(
                    after, references_before[ordinal],
                    "the selected branch reference must move"
                );
            } else {
                assert_eq!(
                    after, references_before[ordinal],
                    "a sibling reference must not move with the selected publication"
                );
            }
        }

        assert_eq!(
            committed.schema_transition_summary().is_some(),
            delta == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
            "only the declared hazard boundary carries schema-transition truth"
        );
        let observed = observe_supply_chain_snapshot(
            execution,
            &world.program,
            &world.handles.for_snapshot(committed.snapshot.clone()),
            &world.runtime,
            &committed.snapshot,
        )
        .expect("the performed branch root is observable through the public exact snapshot");
        compare(
            &expected_supply_chain_branch(&world.program, delta.branch(), Some(delta)),
            &observed,
        )
        .unwrap_or_else(|mismatch| {
            panic!("production delta {delta:?} diverged from the independent oracle: {mismatch:?}")
        });
        world
            .runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .expect("the exact commit snapshot closes once comparison completes");

        let cost = world.runtime.observe_mvcc_cost(&selected_scope).unwrap();
        let delta_cost = cost.sharing_cost_delta();
        assert_eq!(delta_cost.transaction_validation_attempts, 1);
        assert_eq!(delta_cost.candidate_preparations, 1);
        assert_eq!(delta_cost.publication_attempts, 1);
        assert_eq!(delta_cost.branch_population_scans, 0);
        assert_eq!(delta_cost.copied_commit_envelopes, 0);
        assert!(delta_cost.publication_touched_region_count > 0);

        let unrelated_cost = world.runtime.observe_mvcc_cost(&unrelated_scope).unwrap();
        assert_eq!(
            unrelated_cost.sharing_cost_delta(),
            worth_relational::facade::inspection::RelationalBranchSharingCostCounters::default(),
            "a branch-local delta must charge no synchronous work to siblings"
        );
        assert_eq!(unrelated_cost.branch_cell_contacts(), 0);
        assert_all_sibling_semantics(execution, &mut world, &scenarios, &committed_deltas);
    }

    assert_v1_sibling_progressed_after_v2_transition(execution, &mut world, &rewire_before);

    let final_sharing = world.runtime.observe_branch_sharing(&identities).unwrap();
    assert_eq!(final_sharing.unique_root_count(), 9);
    assert_eq!(final_sharing.unique_canonical_commit_artifacts(), 9);
    assert_eq!(final_sharing.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(final_sharing.copied_commit_envelopes(), 0);
    assert_main_remains_operating(execution, &mut world);
}
