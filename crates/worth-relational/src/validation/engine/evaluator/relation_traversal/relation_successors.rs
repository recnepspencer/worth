use std::collections::BTreeSet;

use crate::transactions::data::EntityReference;
use crate::validation::data::InvariantClass;

use super::super::super::context::InvariantExecutionContext;
use super::super::super::request::PreparedRelationIntegrityScope;
use super::planned_successors::PlannedSuccessorMap;

pub(super) struct PreparedSuccessorTraversal<'scope> {
    pub(super) scope: &'scope PreparedRelationIntegrityScope,
    pub(super) class: InvariantClass,
    pub(super) planned_successors: &'scope PlannedSuccessorMap,
}

impl PreparedSuccessorTraversal<'_> {
    pub(super) fn successors(
        &self,
        entity_id: &EntityReference,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Vec<EntityReference> {
        let mut successors = BTreeSet::new();
        self.collect(
            self.planned_successors.get(entity_id),
            &mut successors,
            context,
        );
        self.collect(
            self.scope.visible_successors.get(entity_id),
            &mut successors,
            context,
        );
        successors.into_iter().collect()
    }

    fn collect(
        &self,
        targets: Option<&Vec<EntityReference>>,
        successors: &mut BTreeSet<EntityReference>,
        context: &InvariantExecutionContext<'_, '_>,
    ) {
        let Some(targets) = targets else {
            return;
        };
        for target in targets {
            if !context.checkpoint(1) {
                return;
            }
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
                return;
            }
            successors.insert(target.clone());
        }
    }
}
