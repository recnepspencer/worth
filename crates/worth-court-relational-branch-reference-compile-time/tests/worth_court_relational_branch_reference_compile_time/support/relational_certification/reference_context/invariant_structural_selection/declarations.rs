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

pub(crate) use std::collections::BTreeSet;
pub(crate) use {std::sync::Arc, std::sync::Mutex};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe as observe_supply_chain, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::RelationId, worth_relational::facade::identity::VersionId};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationMutationIntent, worth_relational::facade::transactions::UpdateRelationEndpointsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

#[derive(Clone, Copy)]
pub(crate) struct SelectionExpectation {
    pub(crate) relation: RelationId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    pub(crate) moved_source: EntityId,
    pub(crate) moved_target: EntityId,
}

#[derive(Default)]
pub(crate) struct ProbeEvidence {
    pub(crate) prepared: usize,
    pub(crate) evaluated: usize,
    pub(crate) last_current_version: Option<VersionId>,
    pub(crate) last_touched_entities: usize,
    pub(crate) last_touched_relations: usize,
    pub(crate) last_traversal_steps: usize,
}

pub(crate) struct StructuralSelectionProbe {
    pub(crate) expectation: Arc<Mutex<Option<SelectionExpectation>>>,
    pub(crate) evidence: Arc<Mutex<ProbeEvidence>>,
}

impl StructuralSelectionProbe {
    pub(crate) const RULE_ID: &'static str = "supply_chain.selected-structural-adjacency";

    pub(crate) fn expectation(&self) -> Option<SelectionExpectation> {
        self.expectation
            .lock()
            .expect("probe expectation lock")
            .as_ref()
            .copied()
    }
}

impl CustomInvariantRule for StructuralSelectionProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        crate::relational_certification::reference_context::invariant_structural_selection::descriptor::selection_descriptor()
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let Some(expected) = self.expectation() else {
            return Ok(());
        };
        assert_selected_views!(
            planner.touched(),
            planner.relations(),
            planner.traversal(),
            expected
        );
        self.evidence.lock().expect("probe evidence lock").prepared += 1;
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let Some(expected) = self.expectation() else {
            return Ok(worth_relational::facade::runtime::CustomInvariantVerdict::Pass);
        };
        assert_selected_views!(
            context.touched(),
            context.relations(),
            context.traversal(),
            expected
        );
        let provenance = context.provenance();
        let mut evidence = self.evidence.lock().expect("probe evidence lock");
        evidence.evaluated += 1;
        evidence.last_current_version = Some(provenance.current_version_id);
        evidence.last_touched_entities = provenance.counts.visible_entity_count();
        evidence.last_touched_relations = provenance.counts.visible_relation_count();
        evidence.last_traversal_steps = provenance.traversal.consumed_steps;
        Ok(worth_relational::facade::runtime::CustomInvariantVerdict::Pass)
    }
}

pub(crate) use {crate::relational_certification::reference_context::invariant_structural_selection::selection_fixture::commit_branch_batch_result};
pub(crate) use {crate::relational_certification::reference_context::invariant_structural_selection::selection_fixture::observe_branch_snapshot};
