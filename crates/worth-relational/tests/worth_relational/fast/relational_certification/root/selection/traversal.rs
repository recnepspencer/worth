pub(crate) use std::sync::Arc;

pub(crate) use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
pub(crate) use super::world::supply_chain::{
    commit_branch_batch, relation_kind_id, snapshot_for_supply_chain_identity, EntityKey,
    EntityKind, ExpectedSupplyChainObservation, RelationKey, RelationKind, SupplyChainScale,
    SupplyChainSemanticHandles,
};
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::identity::{EntityId, KindId, RelationId};
pub(crate) use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
pub(crate) use worth_relational::facade::runtime::RelationalRuntime;
pub(crate) use worth_relational::facade::snapshots::SnapshotHandle;
pub(crate) use worth_relational::facade::transactions::{
    EntityReference, MutationIntent, RelationMutationIntent, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
};

pub(crate) fn edge_movement(
    handles: &SupplyChainSemanticHandles,
    baseline: &ExpectedSupplyChainObservation,
) -> EdgeMovement {
    let relation_key = RelationKey::new(RelationKind::CallAtPort, 1);
    let oracle_edge = baseline.relations[&relation_key];
    let edge = EdgeMovement {
        relation_id: handles.relations[&relation_key].id,
        kind_id: relation_kind_id(relation_key.kind),
        ancestor_source: handles.entities[&oracle_edge.source].id,
        ancestor_target: handles.entities[&oracle_edge.target].id,
        main_source: handles.entities[&EntityKey::new(EntityKind::PortCall, 0)].id,
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
        edge.ancestor_probe(Direction::Outgoing, 1),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(Direction::Incoming, 2),
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
        source: EntityReference::Existing(edge.main_source),
        target: EntityReference::Existing(edge.main_target),
    };
    let batch = WorkerIntentBatch::new("phase5-adversarial-main-edge-rewire").push(
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(intent)),
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
        edge.main_probe(Direction::Outgoing, 3),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(Direction::Incoming, 4),
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
        edge.ancestor_probe(Direction::Outgoing, 5),
    );
    let incoming = traverse(
        execution,
        runtime,
        snapshot,
        edge.ancestor_probe(Direction::Incoming, 6),
    );
    assert!(outgoing.relations.contains(&edge.ancestor_tuple()));
    assert!(incoming.relations.contains(&edge.ancestor_tuple()));

    let main_source = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(Direction::Outgoing, 7),
    );
    assert!(main_source.entities.contains(&edge.main_source));
    assert!(!main_source.contains_relation(edge.relation_id));
    let main_target = traverse(
        execution,
        runtime,
        snapshot,
        edge.main_probe(Direction::Incoming, 8),
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
    const fn ancestor_tuple(&self) -> (RelationId, EntityId, EntityId) {
        (self.relation_id, self.ancestor_source, self.ancestor_target)
    }

    const fn main_tuple(&self) -> (RelationId, EntityId, EntityId) {
        (self.relation_id, self.main_source, self.main_target)
    }

    const fn ancestor_probe(&self, direction: Direction, ordinal: u128) -> TraversalProbe {
        let seed = match direction {
            Direction::Outgoing => self.ancestor_source,
            Direction::Incoming => self.ancestor_target,
        };
        TraversalProbe::new(seed, self.kind_id, direction, ordinal)
    }

    const fn main_probe(&self, direction: Direction, ordinal: u128) -> TraversalProbe {
        let seed = match direction {
            Direction::Outgoing => self.main_source,
            Direction::Incoming => self.main_target,
        };
        TraversalProbe::new(seed, self.kind_id, direction, ordinal)
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
    const fn new(seed: EntityId, kind_id: KindId, direction: Direction, ordinal: u128) -> Self {
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
    let seeds = Arc::from([probe.seed]);
    let relation_kind_scope = Some(Arc::from([probe.kind_id]));
    let scope = match probe.direction {
        Direction::Outgoing => QueryScope::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        },
        Direction::Incoming => QueryScope::IncomingNeighborhood {
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
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
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
