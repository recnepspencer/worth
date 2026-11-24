//! Authored endpoint intent gates canonical full-generation membership effects.
use branch::history::access::relevant_writes::generation::AuthoredWriteGeneration;
use branch::history::access::relevant_writes::posting_key::AuthoredWritePostingKey;
use branch::history::access::relevant_writes::selection::RelationalAuthoredWriteSelector as Key;
use branch::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use branch::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouch as Touch;
use branch::history::data::canonical_commit_envelope::descriptive_touches::RelationalTouchAdjacencyDirection as Direction;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::identity::data::record_ids::RelationId;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest, MapKernelContext};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

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
            branch::history::access::relevant_writes::generation::AdjacencyHistoryGap {
                commit: envelope.commit.commit_id,
                relation: None,
                kind: None,
                reason: "unknown-touch-graph",
            },
        );
    };
    let mut affected = authored_relations(execution, context, envelope, touches)?;
    for touch in touches {
        branch::history::access::relevant_writes::visit(context)?;
        let Touch::RelationMembership {
            relation,
            kind,
            source,
            target,
        } = touch
        else {
            continue;
        };
        branch::history::access::relevant_writes::ordered_visit(context, affected.len())?;
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
                branch::history::access::relevant_writes::generation::AdjacencyHistoryGap {
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
                &AuthoredWritePostingKey::Ordinary(Key::Adjacency {
                    anchor,
                    kind: *kind,
                    direction,
                }),
                envelope.commit.commit_id,
            )?;
        }
    }
    for (relation, covered) in affected.iter() {
        branch::history::access::relevant_writes::visit(context)?;
        if !covered {
            // Dense patch detail cannot supply missing full generations.
            // Preserve the actual offending relation and any known kind,
            // but refuse globally: kind alone cannot establish its endpoints.
            let kind = uncovered_kind(context, envelope, touches, *relation)?;
            return incomplete(
                generation,
                execution,
                context,
                branch::history::access::relevant_writes::generation::AdjacencyHistoryGap {
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
        branch::history::access::relevant_writes::visit(context)?;
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
        branch::history::access::relevant_writes::visit(context)?;
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
) -> Result<Option<contracts::identity::data::primitives::KindId>, Denial> {
    for touch in touches {
        branch::history::access::relevant_writes::visit(context)?;
        if let Touch::RelationLifecycle {
            relation: actual,
            kind,
        } = touch
            && *actual == relation
        {
            return Ok(Some(*kind));
        }
    }
    for patch in &envelope.patch.authoritative_record_patches {
        branch::history::access::relevant_writes::visit(context)?;
        if patch.target == RecordRef::Relation(relation)
            && let Some(birth) = patch.relation_birth
        {
            return Ok(Some(birth.kind));
        }
    }
    for intent in &envelope.merged_plan.merged_intents {
        branch::history::access::relevant_writes::visit(context)?;
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
    branch::history::access::relevant_writes::ordered_visit(context, affected.len())?;
    affected.insert_cloned_native_for_request(&relation, &covered, execution)?;
    Ok(())
}

fn incomplete(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    gap: branch::history::access::relevant_writes::generation::AdjacencyHistoryGap,
) -> Result<(), Denial> {
    branch::history::access::relevant_writes::ordered_visit(
        context,
        generation.incomplete_adjacency.len(),
    )?;
    generation
        .incomplete_adjacency
        .insert_cloned_native_for_request(&generation.ordinal, &gap, execution)?;
    Ok(())
}
