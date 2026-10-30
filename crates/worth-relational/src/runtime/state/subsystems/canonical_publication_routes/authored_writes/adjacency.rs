//! Authored endpoint intent gates canonical full-generation membership effects.
use super::{AuthoredWriteGeneration, Denial, Key};
use crate::history::data::{
    CanonicalCommitEnvelope, RelationalDescriptiveTouch as Touch,
    RelationalTouchAdjacencyDirection as Direction,
};
use crate::identity::data::RelationId;
use crate::transactions::data::{MutationIntent, RecordRef, RelationMutationIntent};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest, MapKernelContext};

pub(super) fn append(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Denial> {
    let Some(touches) = envelope.descriptive_touches().exact_touches() else {
        return incomplete(
            generation,
            execution,
            context,
            super::AdjacencyHistoryGap {
                commit: envelope.commit.commit_id,
                relation: None,
                kind: None,
                reason: "unknown-touch-graph",
            },
        );
    };
    let mut affected = authored_relations(execution, context, envelope, touches)?;
    for touch in touches {
        crate::history::visit_authored_write(context)?;
        let Touch::RelationMembership {
            relation,
            kind,
            source,
            target,
        } = touch
        else {
            continue;
        };
        crate::history::visit_authored_write_lookup(
            context,
            affected.len(),
            std::mem::size_of::<RelationId>() as u64,
        )?;
        if affected.get(relation).is_none() {
            continue;
        }
        if relation.generation_value() == 0
            || source.generation_value() == 0
            || target.generation_value() == 0
        {
            return incomplete(
                generation,
                execution,
                context,
                super::AdjacencyHistoryGap {
                    commit: envelope.commit.commit_id,
                    relation: Some(*relation),
                    kind: Some(*kind),
                    reason: "wildcard-membership-generation",
                },
            );
        }
        mark(&mut affected, execution, context, *relation, true)?;
        // Every old/new coordinate contributes BOTH roles. The unchanged role
        // still observes an authored endpoint operation, including a no-op.
        for (anchor, direction) in [
            (*source, Direction::Outgoing),
            (*target, Direction::Incoming),
        ] {
            generation.append(
                execution,
                context,
                &Key::Adjacency {
                    anchor,
                    kind: *kind,
                    direction,
                },
                envelope.commit.commit_id,
            )?;
        }
    }
    for (relation, covered) in affected.iter() {
        crate::history::visit_authored_write(context)?;
        if !covered {
            // Dense patch detail cannot supply missing full generations.
            // Preserve the actual offending relation and any known kind,
            // but refuse globally: kind alone cannot establish its endpoints.
            let kind = uncovered_kind(context, envelope, touches, *relation)?;
            return incomplete(
                generation,
                execution,
                context,
                super::AdjacencyHistoryGap {
                    commit: envelope.commit.commit_id,
                    relation: Some(*relation),
                    kind,
                    reason: "uncovered-relation-lifecycle-or-intent",
                },
            );
        }
    }
    Ok(())
}

fn authored_relations(
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
    touches: &[Touch],
) -> Result<ExecutionOrderedMap<RelationId, bool>, Denial> {
    // Temporary original-request storage joins actual authored relation IDs to
    // canonical effects. It is neither retained authority nor a second index.
    let mut affected = ExecutionOrderedMap::<RelationId, bool>::default();
    for allocation in envelope.record_allocations() {
        if let RecordRef::Relation(relation) = allocation.record() {
            mark(&mut affected, execution, context, *relation, false)?;
        }
    }
    for intent in &envelope.merged_plan.merged_intents {
        crate::history::visit_authored_write(context)?;
        let relation = match intent {
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(value)) => {
                value.relation_id
            }
            MutationIntent::Relation(RelationMutationIntent::Delete(value)) => value.relation_id,
            MutationIntent::Materialization(value) => match value.record() {
                RecordRef::Relation(relation) => relation,
                _ => continue,
            },
            // ApplyAspectPatch retains endpoints despite RelationUpdated's
            // descriptive membership effects. It is a separate field write.
            _ => continue,
        };
        mark(&mut affected, execution, context, relation, false)?;
    }
    for touch in touches {
        crate::history::visit_authored_write(context)?;
        if let Touch::RelationLifecycle { relation, .. } = touch {
            // Includes cascade retirement, not merely explicit relation intents.
            mark(&mut affected, execution, context, *relation, false)?;
        }
    }
    Ok(affected)
}

fn uncovered_kind(
    context: &mut MapKernelContext<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
    touches: &[Touch],
    relation: RelationId,
) -> Result<Option<crate::identity::data::KindId>, Denial> {
    for touch in touches {
        crate::history::visit_authored_write(context)?;
        if let Touch::RelationLifecycle {
            relation: actual,
            kind,
        } = touch
            && *actual == relation
        {
            return Ok(Some(*kind));
        }
    }
    for intent in &envelope.merged_plan.merged_intents {
        crate::history::visit_authored_write(context)?;
        if let MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(value)) = intent
            && value.relation_id == relation
        {
            return Ok(Some(value.kind_id));
        }
    }
    Ok(None)
}

fn mark(
    affected: &mut ExecutionOrderedMap<RelationId, bool>,
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    relation: RelationId,
    covered: bool,
) -> Result<(), Denial> {
    crate::history::visit_authored_write_lookup(
        context,
        affected.len(),
        std::mem::size_of::<RelationId>() as u64,
    )?;
    affected.insert_cloned_native_for_request(&relation, &covered, execution)?;
    Ok(())
}

fn incomplete(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    gap: super::AdjacencyHistoryGap,
) -> Result<(), Denial> {
    crate::history::visit_authored_write_lookup(context, generation.incomplete_adjacency.len(), 8)?;
    generation
        .incomplete_adjacency
        .insert_cloned_native_for_request(&generation.ordinal, &gap, execution)?;
    Ok(())
}
