use std::collections::BTreeSet;

use crate::transactions::data::EntityReference;
use crate::validation::data::InvariantClass;

use super::super::super::context::InvariantExecutionContext;
use super::super::super::request::PreparedRelationIntegrityScope;
use super::planned_successors::PlannedSuccessorMap;

pub(super) struct PreparedSuccessorTraversal<'scope> {
    pub(super) scope: &'scope PreparedRelationIntegrityScope,
    pub(super) class: InvariantClass,
    pub(super) kind: crate::identity::data::KindId,
    pub(super) planned_successors: Option<&'scope PlannedSuccessorMap>,
}

impl PreparedSuccessorTraversal<'_> {
    pub(super) fn successors(
        &self,
        entity_id: &EntityReference,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Vec<EntityReference> {
        let mut successors = BTreeSet::new();
        // Explicit complete certification is a read traversal, not construction
        // of a whole-model scratch graph. Ordinary commits use the prepared
        // change-selected witness below and cannot fall back to this branch.
        if self.scope.complete_source_set
            && let EntityReference::Existing(id) = entity_id
        {
            let view = context.state_view();
            let Ok(relations) = view.try_kind_relation_candidates(
                *id,
                self.kind,
                crate::storage::partition::AdjacencyDirection::Outgoing,
                |units, bytes| {
                    (context.checkpoint(units) && context.claim_scratch(bytes))
                        .then_some(())
                        .ok_or(())
                },
            ) else {
                return Vec::new();
            };
            for relation in relations {
                context.metrics().count_relation_slot_scans(1);
                if !context.checkpoint(1) {
                    return Vec::new();
                }
                if let Some(edge) = view.relation_metadata(relation)
                    && edge.kind_id == self.kind
                    && edge.source == *id
                {
                    let next = EntityReference::Existing(edge.target);
                    if !context.claim_scratch(std::mem::size_of::<EntityReference>() as u64) {
                        return Vec::new();
                    }
                    successors.insert(next);
                }
            }
        }

        self.collect(
            self.planned_successors
                .and_then(|planned| planned.get(entity_id)),
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
            context.metrics().count_relation_slot_scans(1);
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
