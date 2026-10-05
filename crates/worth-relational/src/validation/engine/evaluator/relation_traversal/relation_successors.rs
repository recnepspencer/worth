use std::collections::BTreeSet;

use crate::transactions::data::EntityReference;
use crate::validation::data::{InvariantClass, InvariantViolation};

use super::super::super::context::InvariantExecutionContext;
use super::super::super::request::PreparedRelationIntegrityScope;
use super::planned_successors::PlannedSuccessorMap;
use super::traversal_budget::{traversal_budget_exceeded_violation, RelationTraversalBudget};

pub(super) struct PreparedSuccessorTraversal<'scope> {
    pub(super) scope: &'scope PreparedRelationIntegrityScope,
    pub(super) class: InvariantClass,
    pub(super) contract_id: &'scope crate::schema::data::ContractId,
    pub(super) relation_kind_id: crate::identity::data::KindId,
    pub(super) planned_successors: &'scope PlannedSuccessorMap,
}

impl PreparedSuccessorTraversal<'_> {
    pub(super) fn successors(
        &self,
        entity_id: &EntityReference,
        traversal_budget: &mut RelationTraversalBudget,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Result<Vec<EntityReference>, InvariantViolation> {
        let mut successors = BTreeSet::new();
        self.collect(
            self.planned_successors.get(entity_id),
            traversal_budget,
            &mut successors,
            context,
        )?;
        self.collect(
            self.scope.visible_successors.get(entity_id),
            traversal_budget,
            &mut successors,
            context,
        )?;
        Ok(successors.into_iter().collect())
    }

    fn collect(
        &self,
        targets: Option<&Vec<EntityReference>>,
        traversal_budget: &mut RelationTraversalBudget,
        successors: &mut BTreeSet<EntityReference>,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Result<(), InvariantViolation> {
        let Some(targets) = targets else {
            return Ok(());
        };
        for target in targets {
            if !context.checkpoint(1) {
                return Ok(());
            }
            traversal_budget.record_relation_scan().map_err(|_| {
                traversal_budget_exceeded_violation(
                    self.class,
                    self.contract_id,
                    self.relation_kind_id,
                    *traversal_budget,
                    self.planned_successors,
                )
            })?;
            let bytes = match target {
                EntityReference::Existing(_) => 0,
                EntityReference::Created(created) => {
                    created.client_key.owned_allocation_capacity_bytes()
                }
            };
            if !context.claim_scratch(
                (std::mem::size_of::<EntityReference>() + 3 * std::mem::size_of::<usize>()) as u64
                    + bytes,
            ) {
                return Ok(());
            }
            successors.insert(target.clone());
        }
        Ok(())
    }
}
