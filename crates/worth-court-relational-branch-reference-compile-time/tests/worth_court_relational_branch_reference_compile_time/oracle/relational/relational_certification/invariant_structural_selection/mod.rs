

#[allow(unused_imports)] use crate::relational_certification::reference_context::invariant_structural_selection::*;
pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::sync::Arc, std::sync::Mutex};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe as observe_supply_chain, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::RelationId, worth_relational::facade::identity::VersionId};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationMutationIntent, worth_relational::facade::transactions::UpdateRelationEndpointsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn custom_invariant_structural_reads_stay_on_child_root_after_main_rewire() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let expectation = std::sync::Arc::new(std::sync::Mutex::new(None));
    let evidence = std::sync::Arc::new(std::sync::Mutex::new(crate::relational_certification::reference_context::invariant_structural_selection::ProbeEvidence::default()));
    let registration = worth_relational::facade::runtime::CustomInvariantRegistration::new(StructuralSelectionProbe {
        expectation: std::sync::Arc::clone(&expectation),
        evidence: std::sync::Arc::clone(&evidence),
    })
    .expect("structural selection probe registers");
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let world =
        compile_supply_chain_baseline_with_custom_invariant(execution, program, registration)
            .expect("Court world compiles with the structural selection probe");

    let relation = world.handles.relations
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage, 0)]
        .id;
    let source = world.handles.medical_cargo().id;
    let target = world.handles.aurora_voyage().id;
    let moved_source = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 2)]
        .id;
    let moved_target = world.handles.entities
        [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 1)]
        .id;
    let baseline = observe_supply_chain(execution, &world).expect("baseline remains observable");
    compare(
        &expected_supply_chain_branch(&world.program, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, None),
        &baseline,
    )
    .expect("production baseline matches the independent oracle");

    let (_, fork_source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(execution, BranchId("medical-hold".to_owned()), fork_source)
        .expect("medical-hold retains the baseline root");
    let child_basis_version =
        head_for_supply_chain_branch(&world.runtime, &BranchId("medical-hold".to_owned()))
            .version_id;

    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        worth_relational::facade::transactions::WorkerIntentBatch::new("phase5-main-rewire-cargo-booking").push(worth_relational::facade::transactions::MutationIntent::Relation(
            worth_relational::facade::transactions::RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: relation_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage),
                source: worth_relational::facade::transactions::EntityReference::Existing(moved_source),
                target: worth_relational::facade::transactions::EntityReference::Existing(moved_target),
            }),
        )),
    );

    *expectation.lock().expect("probe expectation lock") = Some(SelectionExpectation {
        relation,
        source,
        target,
        moved_source,
        moved_target,
    });

    let child_before = observe_branch_snapshot(
        execution,
        &world.program,
        &world.runtime,
        &world.handles,
        "medical-hold",
    );
    let main_after = observe_branch_snapshot(
        execution,
        &world.program,
        &world.runtime,
        &world.handles,
        "main",
    );
    assert_eq!(
        child_before.relations
            [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage, 0,)]
            .source,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0)
    );
    assert_eq!(
        main_after.relations
            [&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage, 0,)]
            .source,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 2)
    );
    assert_ne!(child_before.relations, main_after.relations);

    let child_batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("medical-hold".to_owned()),
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
    )
    .expect("the child medical-hold delta lowers from its selected root");
    let child_commit = commit_branch_batch_result(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        child_batch,
    );

    let child_after = observe_branch_snapshot(
        execution,
        &world.program,
        &world.runtime,
        &world.handles,
        "medical-hold",
    );
    compare(
        &expected_supply_chain_branch(
            &world.program,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold,
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo),
        ),
        &child_after,
    )
    .expect("child production state matches the independent medical-hold oracle");

    let custom_execution = child_commit
        .invariant_executions()
        .iter()
        .find_map(|execution| {
            execution.results().iter().find_map(|result| {
                matches!(
                    &result.rule,
                    worth_relational::facade::runtime::InvariantReportedRule::Custom(identity)
                        if identity.rule_id.as_str() == crate::relational_certification::reference_context::invariant_structural_selection::StructuralSelectionProbe::RULE_ID
                )
                .then_some((execution, result))
            })
        })
        .expect("child commit records the structural selection probe");
    assert_eq!(
        custom_execution
            .1
            .custom_provenance()
            .expect("custom execution carries provenance")
            .current_version_id,
        child_basis_version
    );

    let evidence = evidence.lock().expect("probe evidence lock");
    assert_eq!(
        evidence.prepared, 1,
        "child scope is prepared once after arming"
    );
    assert_eq!(
        evidence.evaluated, 1,
        "child scope is evaluated once after arming"
    );
    assert_eq!(evidence.last_current_version, Some(child_basis_version));
    assert_eq!(evidence.last_touched_entities, 2);
    assert_eq!(evidence.last_touched_relations, 1);
    assert!(
        evidence.last_traversal_steps >= 2,
        "both bounded structural walks must charge at least the selected relation"
    );
}

macro_rules! assert_selected_views {
    ($touched:expr, $relations:expr, $traversal:expr, $expected:expr) => {{
        let touched = $touched;
        let relations = $relations;
        let traversal = $traversal;
        let expected = $expected;
        let mut expected_entities = vec![expected.source, expected.target];
        expected_entities.sort_unstable();
        assert_eq!(touched.visible_entity_ids(), expected_entities.as_slice());
        assert_eq!(touched.visible_relation_ids(), &[expected.relation]);
        let mut expected_partitions = vec![
            expected.source.partition_id,
            expected.target.partition_id,
            expected.relation.partition_id,
        ];
        expected_partitions.sort_unstable();
        expected_partitions.dedup();
        assert_eq!(touched.touched_partitions(), expected_partitions.as_slice());
        assert!(
            !touched
                .visible_entity_ids()
                .contains(&expected.moved_source),
            "selected touched scope must exclude main-only moved source"
        );
        assert!(
            !touched
                .visible_entity_ids()
                .contains(&expected.moved_target),
            "selected touched scope must exclude main-only moved target"
        );
        let record = relations
            .relation(expected.relation)
            .expect("selected relation remains visible");
        assert_eq!(record.source, expected.source);
        assert_eq!(record.target, expected.target);
        assert!(relations
            .outgoing_relations_for_entity(expected.source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(relations
            .incoming_relations_for_entity(expected.target)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(relations
            .all_relations_for_entity(expected.source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(!relations
            .outgoing_relations_for_entity(expected.moved_source)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        assert!(!relations
            .incoming_relations_for_entity(expected.moved_target)
            .expect("selected adjacency stays within budget")
            .contains(&expected.relation));
        let outgoing = traversal
            .walk_outgoing_from(&[expected.source], 1)
            .expect("selected outgoing traversal remains bounded");
        assert!(outgoing.traversed_relations().contains(&expected.relation));
        assert!(outgoing.visited_entities().contains(&expected.target));
        let incoming = traversal
            .walk_incoming_from(&[expected.target], 1)
            .expect("selected incoming traversal remains bounded");
        assert!(incoming.traversed_relations().contains(&expected.relation));
        assert!(incoming.visited_entities().contains(&expected.source));
    }};
}


pub(crate) use {crate::relational_certification::reference_context::invariant_structural_selection::selection_fixture::commit_branch_batch_result};
pub(crate) use {crate::relational_certification::reference_context::invariant_structural_selection::selection_fixture::observe_branch_snapshot};
