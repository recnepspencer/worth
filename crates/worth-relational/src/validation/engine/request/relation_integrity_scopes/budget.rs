use std::collections::{BTreeMap, BTreeSet};

use crate::config::data::RelationIntegrityScopeBudget;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::validation::data::{
    InvariantExecutionPoint, InvariantViolation, InvariantViolationFields,
};

use super::PreparedRelationIntegrityScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RelationIntegrityScopeBudgetSnapshot {
    relation_kind_count: usize,
    touched_entity_count: usize,
    deleted_entity_count: usize,
    scanned_relation_count: usize,
    planned_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedRelationIntegrityScopeBudgetExceeded {
    limit_name: &'static str,
    limit: usize,
    observed: usize,
    snapshot: RelationIntegrityScopeBudgetSnapshot,
}

impl PreparedRelationIntegrityScopeBudgetExceeded {
    pub(in crate::validation::engine::request) fn into_violation(
        self,
        execution_point: InvariantExecutionPoint,
    ) -> InvariantViolation {
        InvariantViolation {
            class: execution_point.class(),
            code: crate::diagnostics::data::DiagnosticCode::PreparationFailure,
            detail: format!(
                "relation integrity scope preparation exceeded '{}' budget: {} > {}",
                self.limit_name, self.observed, self.limit
            ),
            fields: InvariantViolationFields::RelationIntegrityScopeBudgetExceeded {
                limit_name: self.limit_name.to_string(),
                limit: self.limit,
                observed: self.observed,
                relation_kind_count: self.snapshot.relation_kind_count,
                touched_entity_count: self.snapshot.touched_entity_count,
                deleted_entity_count: self.snapshot.deleted_entity_count,
                scanned_relation_count: self.snapshot.scanned_relation_count,
                planned_edge_count: self.snapshot.planned_edge_count,
            },
        }
    }
}

pub(super) fn scope_budget_snapshot(
    scopes: &BTreeMap<KindId, PreparedRelationIntegrityScope>,
    touched_entities: &BTreeSet<EntityId>,
    created_candidate_count: usize,
    deleted_entities: &BTreeSet<EntityId>,
    scanned_relations: &BTreeSet<RelationId>,
    planned_edge_count: usize,
) -> RelationIntegrityScopeBudgetSnapshot {
    RelationIntegrityScopeBudgetSnapshot {
        relation_kind_count: scopes.len(),
        touched_entity_count: touched_entities.len() + created_candidate_count,
        deleted_entity_count: deleted_entities.len(),
        scanned_relation_count: scanned_relations.len(),
        planned_edge_count,
    }
}

pub(super) fn ensure_relation_integrity_scope_budget(
    budget: &RelationIntegrityScopeBudget,
    snapshot: RelationIntegrityScopeBudgetSnapshot,
) -> Result<(), PreparedRelationIntegrityScopeBudgetExceeded> {
    let checks = [
        (
            "max_relation_kinds",
            budget.max_relation_kinds,
            snapshot.relation_kind_count,
        ),
        (
            "max_touched_entities",
            budget.max_touched_entities,
            snapshot.touched_entity_count,
        ),
        (
            "max_deleted_entities",
            budget.max_deleted_entities,
            snapshot.deleted_entity_count,
        ),
        (
            "max_scanned_relations",
            budget.max_scanned_relations,
            snapshot.scanned_relation_count,
        ),
        (
            "max_planned_edges",
            budget.max_planned_edges,
            snapshot.planned_edge_count,
        ),
    ];
    for (limit_name, limit, observed) in checks {
        if observed > limit {
            return Err(PreparedRelationIntegrityScopeBudgetExceeded {
                limit_name,
                limit,
                observed,
                snapshot,
            });
        }
    }
    Ok(())
}
