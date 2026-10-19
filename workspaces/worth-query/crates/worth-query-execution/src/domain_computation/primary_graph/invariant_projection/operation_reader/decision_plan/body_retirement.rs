//! Complete outer membership and every inner selection are observed before retiring anything.
use super::{
    ApplicationSchema, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    application_contribution::QueryRequestExecution, operation_body::PreparedRetirementBatch,
    HandlerExecutionDenial, WorthQueryAdvancementDenial,
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryInvariantMutationTarget, WorthQueryManagedComputationResourceDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyOwnedSelectionRetirement as Rule,
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Source,
    ApplicationOperationDecisionReadTarget as Read,
};
impl<S: ApplicationSchema, O>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    pub(in crate::domain_computation::primary_graph) fn body_retirement<E>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<S, E>,
        rule: &Rule,
        index: usize,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRetirementBatch<S>, HandlerExecutionDenial> {
        let source = Source::RelatedItemCount {
            relation: rule.owners_relation.clone(),
            from: rule.scope.clone(),
            to: rule.owner.clone(),
            direction: Direction::Outgoing,
        };
        let owners = self.observed_requirement_collection(root, &source, execution, context)?;
        if owners.ids.len() > rule.maximum_owners.get() {
            return Err(HandlerExecutionDenial::new(Denial::new(
                Kind::CandidateCapacityExceeded,
                &rule.owners_relation,
            )));
        }
        let per_target = size_of::<(
            WorthQueryInvariantMutationTarget<S, ()>,
            WorthQueryInvariantMutationTarget<S, ()>,
        )>()
        .checked_add(
            rule.owner
                .len()
                .checked_add(rule.selected.len())
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(overflow)?,
        )
        .and_then(|n| n.checked_add(4 * size_of::<usize>()))
        .ok_or_else(overflow)?;
        let bytes = owners
            .ids
            .len()
            .checked_mul(per_target)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(overflow)?;
        let memory = execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
        let selection = Read::Relation {
            relation: rule.selection_relation.clone(),
            from: rule.owner.clone(),
            to: rule.selected.clone(),
        };
        let mut targets = Vec::with_capacity(owners.ids.len());
        for owner_id in &owners.ids {
            let owner = self.body_observed_selection(*owner_id, &rule.owner)?;
            if let Some(selected) =
                self.body_related_selection(&owner.identity, &selection, execution, context)?
            {
                targets.push((owner.target, selected.target));
            }
        }
        Ok(PreparedRetirementBatch {
            rule: index,
            targets,
            _memory: memory,
        })
    }
}
fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
