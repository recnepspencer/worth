pub(crate) use std::sync::Arc;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::RelationId};
pub(crate) use {worth_relational::facade::query::DeterministicQueryPlanKey, worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::QueryAccessContract, worth_relational::facade::query::QueryExecutionShape, worth_relational::facade::query::QueryLocalityClass, worth_relational::facade::query::QueryOrderingContract, worth_relational::facade::query::QueryScope, worth_relational::facade::query::ReductionDiscipline};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::snapshots::SnapshotHandle};
pub(crate) use {worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationMutationIntent, worth_relational::facade::transactions::UpdateRelationEndpointsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn edge_movement(
    handles: &SupplyChainSemanticHandles,
    baseline: &ExpectedSupplyChainObservation,
) -> EdgeMovement {
    let relation_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let oracle_edge = baseline.relations[&relation_key];
    let edge = EdgeMovement {
        relation_id: handles.relations[&relation_key].id,
        kind_id: relation_kind_id(relation_key.kind),
        ancestor_source: handles.entities[&oracle_edge.source].id,
        ancestor_target: handles.entities[&oracle_edge.target].id,
        main_source: handles.entities[&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 0)].id,
        main_target: handles.rewire_port().id,
    };
    assert_ne!(edge.ancestor_source, edge.main_source);
    assert_ne!(edge.ancestor_target, edge.main_target);
    edge
}

pub(crate) fn assert_ancestor_edge_is_visible(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    edge: &EdgeMovement,
) {
    let outgoing = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing, 1),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming, 2),
    );
    assert!(outgoing.relations.contains(&edge.ancestor_tuple()));
    assert!(incoming.relations.contains(&edge.ancestor_tuple()));
}

pub(crate) fn publish_main_rewire(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    edge: &EdgeMovement,
) {
    let intent = UpdateRelationEndpointsIntent {
        relation_id: edge.relation_id,
        kind_id: edge.kind_id,
        source: worth_relational::facade::transactions::EntityReference::Existing(edge.main_source),
        target: worth_relational::facade::transactions::EntityReference::Existing(edge.main_target),
    };
    let batch = worth_relational::facade::transactions::WorkerIntentBatch::new(
        "phase5-adversarial-main-edge-rewire",
    )
    .push(
        worth_relational::facade::transactions::MutationIntent::Relation(
            worth_relational::facade::transactions::RelationMutationIntent::UpdateEndpoints(intent),
        ),
    );
    commit_branch_batch(execution, runtime, BranchId("main".to_owned()), batch);
}

pub(crate) fn assert_main_edge_is_visible(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    edge: &EdgeMovement,
) {
    let outgoing = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing, 3),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming, 4),
    );
    assert!(outgoing.relations.contains(&edge.main_tuple()));
    assert!(incoming.relations.contains(&edge.main_tuple()));
}

pub(crate) fn assert_child_retains_ancestor_and_rejects_main_edge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    edge: &EdgeMovement,
) {
    let outgoing = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing, 5),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming, 6),
    );
    assert!(outgoing.relations.contains(&edge.ancestor_tuple()));
    assert!(incoming.relations.contains(&edge.ancestor_tuple()));

    let main_source = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing, 7),
    );
    assert!(main_source.entities.contains(&edge.main_source));
    assert!(!main_source.contains_relation(edge.relation_id));
    let main_target = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming, 8),
    );
    assert!(main_target.entities.contains(&edge.main_target));
    assert!(!main_target.contains_relation(edge.relation_id));
}

pub(crate) struct EdgeMovement {
    pub(crate) relation_id: RelationId,
    pub(crate) kind_id: KindId,
    pub(crate) ancestor_source: EntityId,
    pub(crate) ancestor_target: EntityId,
    pub(crate) main_source: EntityId,
    pub(crate) main_target: EntityId,
}

impl EdgeMovement {
    pub(crate) const fn ancestor_tuple(&self) -> (RelationId, EntityId, EntityId) {
        (self.relation_id, self.ancestor_source, self.ancestor_target)
    }

    pub(crate) const fn main_tuple(&self) -> (RelationId, EntityId, EntityId) {
        (self.relation_id, self.main_source, self.main_target)
    }

    pub(crate) const fn ancestor_probe(
        &self,
        direction: Direction,
        ordinal: u128,
    ) -> TraversalProbe {
        let seed = match direction {
            crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing => self.ancestor_source,
            crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming => self.ancestor_target,
        };
        crate::relational_certification::ordinary_context::root_branch_traversal_isolation::TraversalProbe::new(seed, self.kind_id, direction, ordinal)
    }

    pub(crate) const fn main_probe(&self, direction: Direction, ordinal: u128) -> TraversalProbe {
        let seed = match direction {
            crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing => self.main_source,
            crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming => self.main_target,
        };
        crate::relational_certification::ordinary_context::root_branch_traversal_isolation::TraversalProbe::new(seed, self.kind_id, direction, ordinal)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Direction {
    Outgoing,
    Incoming,
}

pub(crate) struct TraversalProbe {
    pub(crate) seed: EntityId,
    pub(crate) kind_id: KindId,
    pub(crate) direction: Direction,
    pub(crate) ordinal: u128,
}

impl TraversalProbe {
    pub(crate) const fn new(
        seed: EntityId,
        kind_id: KindId,
        direction: Direction,
        ordinal: u128,
    ) -> Self {
        Self {
            seed,
            kind_id,
            direction,
            ordinal,
        }
    }
}

pub(crate) struct TraversalObservation {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) relations: Vec<(RelationId, EntityId, EntityId)>,
}

impl TraversalObservation {
    pub(crate) fn contains_relation(&self, relation_id: RelationId) -> bool {
        self.relations
            .iter()
            .any(|relation| relation.0 == relation_id)
    }
}

pub(crate) fn traverse(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    probe: TraversalProbe,
) -> TraversalObservation {
    let context = runtime
        .read_truth()
        .query_plan_context(snapshot)
        .expect("root-qualified query context remains available");
    let seeds = std::sync::Arc::from([probe.seed]);
    let relation_kind_scope = Some(std::sync::Arc::from([probe.kind_id]));
    let scope = match probe.direction {
        crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Outgoing => worth_relational::facade::query::QueryScope::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        },
        crate::relational_certification::ordinary_context::root_branch_traversal_isolation::Direction::Incoming => worth_relational::facade::query::QueryScope::IncomingNeighborhood {
            seeds,
            relation_kind_scope,
        },
    };
    let packet = traversal_packet(context, scope, probe.ordinal);
    let plan = runtime
        .read_truth()
        .plan_query_packet(snapshot, packet)
        .expect("root-qualified traversal plan is admitted");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("root-qualified traversal executes");
    TraversalObservation {
        entities: outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect(),
        relations: outcome
            .result
            .relations
            .iter()
            .map(|record| (record.relation_id, record.source, record.target))
            .collect(),
    }
}

pub(crate) fn traversal_packet(
    context_id: worth_relational::facade::query::QueryPlanContextId,
    scope: QueryScope,
    ordinal: u128,
) -> PlannedQueryPacket {
    PlannedQueryPacket {
        label: format!("phase5-branch-root-traversal-{ordinal}"),
        context_id,
        scope,
        locality: worth_relational::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: worth_relational::facade::query::QueryOrderingContract::CanonicalTraversalOrder,
        access_contract:
            worth_relational::facade::query::QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: worth_relational::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: worth_relational::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_171_500 + ordinal),
        target_count_hint: 1,
    }
}

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) -> SnapshotHandle {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .expect("fork retains the exact ancestor root");
    current_snapshot(runtime, branch)
}

pub(crate) fn current_snapshot(runtime: &RelationalRuntime, branch: &str) -> SnapshotHandle {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
}
