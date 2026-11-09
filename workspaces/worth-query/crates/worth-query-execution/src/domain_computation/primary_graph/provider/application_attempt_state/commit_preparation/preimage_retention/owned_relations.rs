//! An observed empty owner adjacency closes intra-roster loci with no possible endpoint.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Decision;
use worth_query_installation::facade::{
    InstalledPreImageRelationLocus, WorthQueryOperationGraphReadScope as Scope,
};
use worth_relational::facade::{mvcc::ValidatedMutationFootprint, transactions::RecordRef};

pub(super) fn is_empty(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    locus: &InstalledPreImageRelationLocus,
    facts: &[Decision],
    mutation: &ValidatedMutationFootprint,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<bool, WorthQueryProviderSessionFailure> {
    let identity = if let Some(pending) = attempt.aftermath_causality() {
        pending.prepared_correction().authored_declaration()
    } else {
        attempt
            .original_authority()
            .retained_governed_input_carrier()
            .and_then(|input| input.correction())
            .and_then(|original| original.association().declaration().authored_identity())
    };
    let Some(identity) = identity else {
        return Ok(false);
    };
    let Some(anchors) = super::authored_anchors::select(attempt)? else {
        return Ok(false);
    };
    for member in identity.co_anchors() {
        let Some(owner) = member.member_owner() else {
            continue;
        };
        if owner.from() != locus.from() && owner.from() != locus.to() {
            continue;
        }
        for (record, entity) in &anchors {
            if *entity != owner.to() || !mutation.records().any(|actual| actual == record) {
                continue;
            }
            let RecordRef::Entity(root) = record else {
                continue;
            };
            for fact in facts {
                checkpoint(context, 1).map_err(|cause| {
                    super::relations::request_stop(worth_execution::WorkCeilingDenial::Stopped(
                        cause,
                    ))
                })?;
                let Decision::Application { read_scope, fact } = fact else {
                    continue;
                };
                if !matches!(read_scope.as_ref(), Scope::Relation(scope) if scope.relation() == owner.relation() && scope.from() == owner.from() && scope.to() == owner.to())
                {
                    continue;
                }
                if matches!(fact.as_ref(), Fact::Adjacency { anchor, direction: Direction::Incoming, relations, .. } if anchor == root && relations.is_empty())
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
