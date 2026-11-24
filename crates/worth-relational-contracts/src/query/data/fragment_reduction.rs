use serde::{Deserialize, Serialize};

use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};

use crate::identity::data::EntityId;
use crate::query::data::planned_packets::{
    DeterministicQueryFragmentKey, DeterministicQueryPlanKey, QueryExecutionShape,
    QueryOrderingContract,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryFragmentCounters {
    pub target_count: usize,
    pub authoritative_entity_records_emitted: usize,
    pub authoritative_relation_records_emitted: usize,
    pub touched_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TraversalEntityVisitKey {
    pub depth: u32,
    pub root_seed: EntityId,
    pub via_relation: Option<crate::identity::data::RelationId>,
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TraversalRelationVisitKey {
    pub depth: u32,
    pub root_seed: EntityId,
    pub relation_id: crate::identity::data::RelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TraversalReductionBasis {
    pub entity_visit_keys: Vec<TraversalEntityVisitKey>,
    pub relation_visit_keys: Vec<TraversalRelationVisitKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueryWorkerFragment {
    pub plan_key: DeterministicQueryPlanKey,
    pub fragment_key: DeterministicQueryFragmentKey,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub counters: QueryFragmentCounters,
    pub traversal_basis: Option<TraversalReductionBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalQueryResult {
    pub execution_shape: QueryExecutionShape,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub reduction_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryComplexitySummary {
    pub packet_count: usize,
    pub fragment_count: usize,
    pub touched_partitions: usize,
    pub target_count: usize,
    pub authoritative_entity_records_emitted: usize,
    pub authoritative_relation_records_emitted: usize,
}
