//! Bounded, metered related-record absence (`NOT EXISTS`) and existence
//! (`EXISTS`) narrowing over index-selected root candidates.
//!
//! Complexity:
//!   Time:  O(candidates * log(candidates) + edges * log(edges)
//!            + inspected_unique_targets * (target_fields * roster_width + target_filter_work))
//!   Space: O(candidates + edges + inspected_unique_targets)
//!
//! Every governance check precedes any edge or row access. Every scan, sort,
//! lookup, and scratch buffer is metered via `SourceFactsAllocation` and
//! `RootSelectionWork` before allocation or traversal. Every examined
//! candidate's adjacency structural revision and every examined target's
//! guarded fields join `evidence` (proven sound in `RelatedAbsence.tla`).
use super::residual::{charge_work, log2_ceil, try_vec_exact};
use super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use crate::domain_computation::primary_graph::application_query::read_execution::{
    read_execution_denial, relational_read_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::{BTreeMap, BTreeSet};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryRelatedPolarity, RelationDirection,
};
use worth_query_installation::facade::WorthQueryInstalledRelatedPredicate;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::RelationalAdjacencyDirection;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

mod target_check;
mod target_equality;
mod target_filter;
use target_check::TargetMatcher;

struct AdmittedPredicate<'a> {
    predicate: &'a WorthQueryInstalledRelatedPredicate,
    relation_kind: KindId,
    target_kind: KindId,
}

struct NarrowingState<'a, 'work> {
    evidence: &'a mut RootPathSourceBuilder,
    work: &'a mut RootSelectionWork<'work>,
}

pub fn narrow<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    mut candidates: Vec<EntityId>,
    evidence: &mut RootPathSourceBuilder,
    work: &mut RootSelectionWork<'_>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let predicates = read.plan.contract.related_predicates();
    // Charge governance admission work and verify governance BEFORE touching
    // any rows or edges, even if `candidates` is empty, so unauthorized
    // principals fail closed with bounded metered work.
    let checks = governance_check_count(predicates)
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, "related_predicate"))?;
    let gov_work = checks
        .checked_mul(log2_ceil(read.plan.governance.internal_rule_count().max(1)))
        .ok_or_else(|| relational_read_denial(Cause::WorkCounterOverflow, "related_predicate"))?;
    charge_work(read.execution, work, gov_work)?;
    for predicate in predicates {
        admit_related_governance(read, predicate)?;
    }
    if candidates.is_empty() {
        return Ok(candidates);
    }
    let mut state = NarrowingState { evidence, work };
    for predicate in predicates {
        if candidates.is_empty() {
            break;
        }
        let admitted = resolve_related_layout(read, predicate)?;
        candidates = narrow_one(read, admitted, candidates, &mut state)?;
    }
    Ok(candidates)
}

fn governance_check_count(predicates: &[WorthQueryInstalledRelatedPredicate]) -> Option<u64> {
    let mut total = 0_u64;
    for predicate in predicates {
        let fields = predicate.target_predicates().len().checked_add(
            predicate
                .target_where_filter()
                .map_or(0, |filter| filter.fields().len()),
        )?;
        total = total
            .checked_add(1)?
            .checked_add(u64::try_from(fields).ok()?)?;
    }
    Some(total)
}

fn admit_related_governance<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    predicate: &WorthQueryInstalledRelatedPredicate,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if !read.plan.governance.admit_internal_relation(
        (predicate.relation(), predicate.from(), predicate.to()),
        predicate.direction(),
        ApplicationQueryObservableInfluence::RowPresence,
    ) {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            predicate.relation(),
        ));
    }
    admit_target_fields(read, predicate)
}

fn resolve_related_layout<'a, ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    predicate: &'a WorthQueryInstalledRelatedPredicate,
) -> Result<AdmittedPredicate<'a>, WorthQueryApplicationReadExecutionDenial> {
    let layout = read
        .graph
        .relation(predicate.relation())
        .filter(|layout| {
            read.graph.entity_kind(predicate.from()) == Some(layout.from)
                && read.graph.entity_kind(predicate.to()) == Some(layout.to)
        })
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
                predicate.relation(),
            )
        })?;
    let target_kind = read
        .graph
        .entity_kind(predicate.target_entity())
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                predicate.target_entity(),
            )
        })?;
    Ok(AdmittedPredicate {
        predicate,
        relation_kind: layout.kind,
        target_kind,
    })
}

fn admit_target_fields<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    predicate: &WorthQueryInstalledRelatedPredicate,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let check = |field: (&str, &str, &str), key| {
        read.plan
            .governance
            .admit_internal_projection(field, key, ApplicationQueryObservableInfluence::RowPresence)
            .ok_or_else(|| {
                read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
                    field.2,
                )
            })
    };
    for target in predicate.target_predicates() {
        check(target.field(), target.field_key())?;
    }
    for field in predicate
        .target_where_filter()
        .map_or(&[][..], |filter| filter.fields())
    {
        check(field.field(), field.field_key())?;
    }
    Ok(())
}

fn narrow_one<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    admitted: AdmittedPredicate<'_>,
    mut candidates: Vec<EntityId>,
    state: &mut NarrowingState<'_, '_>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let subject = admitted.predicate.relation();
    let stop = |cause| relational_read_denial(cause, subject);
    state.work.checkpoint(subject)?;
    let allocation = SourceFactsAllocation::open(read.execution).map_err(stop)?;
    let matcher = TargetMatcher::new(
        read,
        admitted.predicate,
        admitted.target_kind,
        &allocation,
        state.work,
    )?;
    let (edges, adj_dir) =
        load_sorted_edges(read, &admitted, &mut candidates, &allocation, state.work)?;
    evaluate_candidates(
        read,
        &admitted,
        &matcher,
        &edges,
        adj_dir,
        candidates,
        &allocation,
        state,
    )
}

fn load_sorted_edges<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    admitted: &AdmittedPredicate<'_>,
    candidates: &mut Vec<EntityId>,
    allocation: &SourceFactsAllocation,
    work: &mut RootSelectionWork<'_>,
) -> Result<
    (Vec<(EntityId, EntityId)>, RelationalAdjacencyDirection),
    WorthQueryApplicationReadExecutionDenial,
> {
    let subject = admitted.predicate.relation();
    let stop = |cause| relational_read_denial(cause, subject);
    let cand_count = candidates.len();
    let frontier_work = (cand_count as u64)
        .checked_mul(log2_ceil(cand_count))
        .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    charge_work(read.execution, work, frontier_work)?;
    allocation
        .claim_tree_copy::<EntityId>(read.execution, cand_count)
        .map_err(stop)?;
    candidates.sort_unstable();
    candidates.dedup();
    let frontier: BTreeSet<EntityId> = candidates.iter().copied().collect();

    work.checkpoint(subject)?;
    let direction = admitted.predicate.direction();
    let (adjacency, adj_dir) = match direction {
        RelationDirection::Forward => (
            read.projection
                .bounded_outgoing_relations_for_frontier(
                    read.execution,
                    &frontier,
                    admitted.relation_kind,
                    None,
                )
                .map_err(stop)?
                .map_err(|_| work_limit_exceeded(subject))?,
            RelationalAdjacencyDirection::Outgoing,
        ),
        RelationDirection::Reverse => (
            read.projection
                .bounded_incoming_relations_for_frontier(
                    read.execution,
                    &frontier,
                    admitted.relation_kind,
                    None,
                )
                .map_err(stop)?
                .map_err(|_| work_limit_exceeded(subject))?,
            RelationalAdjacencyDirection::Incoming,
        ),
    };
    work.charge(
        adjacency.adjacency_lists_read(),
        adjacency.relation_records_examined(),
        adjacency.endpoint_records_reserved(),
        subject,
    )?;
    let records = adjacency.into_records();
    let edge_count = records.len();
    let edge_work = (edge_count as u64)
        .checked_mul(log2_ceil(edge_count).saturating_add(1))
        .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
    charge_work(read.execution, work, edge_work)?;
    allocation
        .claim_slice::<(EntityId, EntityId)>(read.execution, edge_count)
        .map_err(stop)?;
    let mut edges: Vec<(EntityId, EntityId)> = try_vec_exact(edge_count)?;
    for record in records {
        let pair = match direction {
            RelationDirection::Forward => (record.source, record.target),
            RelationDirection::Reverse => (record.target, record.source),
        };
        edges.push(pair);
    }
    edges.sort_unstable();
    edges.dedup();
    Ok((edges, adj_dir))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_candidates<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    admitted: &AdmittedPredicate<'_>,
    matcher: &TargetMatcher<'_>,
    edges: &[(EntityId, EntityId)],
    adj_dir: RelationalAdjacencyDirection,
    candidates: Vec<EntityId>,
    allocation: &SourceFactsAllocation,
    state: &mut NarrowingState<'_, '_>,
) -> Result<Vec<EntityId>, WorthQueryApplicationReadExecutionDenial> {
    let subject = admitted.predicate.relation();
    let stop = |cause| relational_read_denial(cause, subject);
    allocation
        .claim_slice::<EntityId>(read.execution, candidates.len())
        .map_err(stop)?;
    let mut retained: Vec<EntityId> = try_vec_exact(candidates.len())?;
    let mut memo: BTreeMap<EntityId, bool> = BTreeMap::new();
    let mut cursor = 0_usize;

    for candidate in candidates {
        state.work.checkpoint(subject)?;
        state.evidence.observe_entity(read.execution, candidate)?;
        state.evidence.observe_adjacency(
            read.execution,
            read.projection,
            candidate,
            admitted.relation_kind,
            adj_dir,
            subject,
            state.work,
        )?;
        while cursor < edges.len() && edges[cursor].0 < candidate {
            cursor += 1;
        }
        let start = cursor;
        while cursor < edges.len() && edges[cursor].0 == candidate {
            cursor += 1;
        }
        let has_match = any_neighbor_matches(
            read,
            subject,
            matcher,
            &edges[start..cursor],
            &mut memo,
            allocation,
            state,
        )?;
        let keep = match admitted.predicate.polarity() {
            ApplicationQueryRelatedPolarity::Absent => !has_match,
            ApplicationQueryRelatedPolarity::Exists => has_match,
        };
        if keep {
            retained.push(candidate);
        }
    }
    Ok(retained)
}

#[allow(clippy::too_many_arguments)]
fn any_neighbor_matches<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    subject: &str,
    matcher: &TargetMatcher<'_>,
    neighbors: &[(EntityId, EntityId)],
    memo: &mut BTreeMap<EntityId, bool>,
    allocation: &SourceFactsAllocation,
    state: &mut NarrowingState<'_, '_>,
) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
    if matcher.is_unfiltered() {
        if let Some(&(_, witness)) = neighbors.first() {
            state.evidence.observe_entity(read.execution, witness)?;
            return Ok(true);
        }
        return Ok(false);
    }
    let stop = |cause| relational_read_denial(cause, subject);
    for &(_, target_id) in neighbors {
        state.work.checkpoint(subject)?;
        let lookup_work = log2_ceil(memo.len().max(1));
        charge_work(read.execution, state.work, lookup_work)?;
        let matched = if let Some(&cached) = memo.get(&target_id) {
            cached
        } else {
            let evaluated = matcher.matches(read, target_id, state.evidence, state.work)?;
            allocation
                .claim_tree::<(EntityId, bool)>(read.execution, memo.len())
                .map_err(stop)?;
            memo.insert(target_id, evaluated);
            evaluated
        };
        if matched {
            return Ok(true);
        }
    }
    Ok(false)
}

fn work_limit_exceeded(subject: &str) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
        subject,
    )
}
