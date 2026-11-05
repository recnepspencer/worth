//! Correlate declared relation material with the actor's actual applied lifecycle.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryRelationPreImageDenial as Cause, WorthQueryRelationPreImageFailure as Failure,
};
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationDecisionFact as Decision, WorthQueryPrimaryGraphLayout,
};
use worth_query_installation::facade::{
    InstalledPreImageDemand, InstalledPreImageRelationLocus, WorthQueryOperationGraphReadScope,
};
use worth_relational::facade::{
    mvcc::{
        ValidatedRelationMutation, ValidatedRelationMutationFootprint, ValidatedRelationalProposal,
    },
    transactions::RecordRef,
};

pub(crate) struct WorthQueryRetainedRelationPreImageSeal {
    rows: Vec<(InstalledPreImageRelationLocus, ValidatedRelationMutation)>,
    loci: Vec<InstalledPreImageRelationLocus>,
    footprint: ValidatedRelationMutationFootprint,
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryRetainedRelationPreImageSeal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<(InstalledPreImageRelationLocus, ValidatedRelationMutation)>,
        Vec<InstalledPreImageRelationLocus>,
        ValidatedRelationMutationFootprint,
        Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        (self.rows, self.loci, self.footprint, self.memory)
    }
}
pub(super) fn retain(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    layout: &WorthQueryPrimaryGraphLayout,
    demand: &InstalledPreImageDemand,
    candidate: &ValidatedRelationalProposal,
    facts: &[Decision],
    retained: WorthQueryRetainedPreImage,
) -> Result<WorthQueryRetainedPreImage, WorthQueryProviderSessionFailure> {
    let kinds: Vec<_> = demand
        .relations()
        .iter()
        .map(|locus| {
            layout
                .relation(locus.relation())
                .map(|relation| relation.kind)
                .ok_or_else(|| {
                    denied(
                        Cause::MissingObservation,
                        retained.target_record().cloned().into_iter().collect(),
                    )
                })
        })
        .collect::<Result<_, _>>()?;
    let footprint = candidate
        .relation_mutation_footprint(execution, &kinds)
        .map_err(|cause| {
            provider_failure(
                WorthQueryProviderSessionProtocolStage::Commit,
                "relation mutation projection refused",
            )
            .with_native_preparation_error(cause)
        })?;
    let quote_work = demand
        .relations()
        .len()
        .checked_mul(footprint.relations().len().saturating_add(1))
        .and_then(|work| {
            demand.relations().iter().try_fold(work, |work, locus| {
                work.checked_add(locus.relation().len())?
                    .checked_add(locus.from().len())?
                    .checked_add(locus.to().len())
            })
        })
        .ok_or_else(|| {
            denied(
                Cause::ExceedsByteBound,
                retained.target_record().cloned().into_iter().collect(),
            )
        })?;
    execution
        .run_native_with_checkpoint(|context| checkpoint(context, quote_work as u64))
        .map_err(request_stop)?
        .0
        .map_err(|cause| request_stop(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    // The seal and retained carrier briefly coexist. Locus strings are moved
    // between them; every repeated row's own strings retain their original grant.
    let rows_bytes = footprint
        .relations()
        .len()
        .checked_mul(2 * size_of::<(InstalledPreImageRelationLocus, ValidatedRelationMutation)>())
        .and_then(|n| {
            demand.relations().iter().try_fold(n, |n, locus| {
                let names = locus
                    .relation()
                    .len()
                    .checked_add(locus.from().len())?
                    .checked_add(locus.to().len())?;
                let kind = layout.relation(locus.relation())?.kind;
                let copies = footprint
                    .relations()
                    .iter()
                    .filter(|mutation| mutation.kind() == kind)
                    .count()
                    .checked_add(1)?;
                n.checked_add(names.checked_mul(copies)?)?
                    .checked_add(size_of::<InstalledPreImageRelationLocus>())
            })
        })
        .ok_or_else(|| {
            denied(
                Cause::ExceedsByteBound,
                retained.target_record().cloned().into_iter().collect(),
            )
        })?;
    let memory = execution
        .allocation_policy()
        .reserve_memory(rows_bytes as u64)
        .map_err(WorthQueryProviderSessionFailure::allocation_denied)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(footprint.relations().len())
        .map_err(|_| {
            provider_failure(
                WorthQueryProviderSessionProtocolStage::Commit,
                "relation preimage allocation failed",
            )
        })?;
    let field_mutations = candidate
        .mutation_footprint(Some(demand))
        .into_projected()
        .expect("requested mutation footprint");
    let mut total = retained.total_encoded_bytes();
    execution
        .run_native_with_checkpoint(|context| {
            for (locus, kind) in demand.relations().iter().zip(kinds) {
                let mut count = 0usize;
                for mutation in footprint.relations() {
                    checkpoint(context, 1).map_err(|cause| {
                        request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                    })?;
                    if mutation.kind() != kind {
                        continue;
                    }
                    // The existing correction owner rejects audit-visible adjacency
                    // support. A new relation inverse demand must refuse that shape
                    // before the original edit, rather than promise an unusable undo.
                    if mutation.is_audit_retained() {
                        return Err(denied(
                            Cause::UnsupportedVisibility,
                            vec![RecordRef::Relation(mutation.relation())],
                        ));
                    }
                    let endpoints = mutation
                        .before()
                        .or_else(|| mutation.after())
                        .expect("actual lifecycle transition");
                    let mut observed = false;
                    for fact in facts {
                        checkpoint(context, observation_work(fact, locus)).map_err(|cause| {
                            request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                        })?;
                        if observes(
                            fact,
                            locus,
                            kind,
                            mutation.relation(),
                            endpoints,
                            mutation.before().is_none(),
                        ) {
                            observed = true;
                            break;
                        }
                    }
                    if !observed {
                        return Err(denied(
                            Cause::MissingObservation,
                            vec![RecordRef::Relation(mutation.relation())],
                        ));
                    }
                    // Full coordinates for the record and both before/after endpoints.
                    total = total
                        .checked_add(16 + 4 + 2 * (1 + 2 * 16))
                        .ok_or_else(|| {
                            denied(
                                Cause::ExceedsByteBound,
                                vec![RecordRef::Relation(mutation.relation())],
                            )
                        })?;
                    if total > demand.maximum_encoded_bytes() {
                        return Err(denied(
                            Cause::ExceedsByteBound,
                            vec![RecordRef::Relation(mutation.relation())],
                        ));
                    }
                    rows.push((locus.clone(), *mutation));
                    count += 1;
                }
                if count == 0 {
                    // An empty historical locus requires an actual complete empty read
                    // on a record this actor mutates; absence of a read is not emptiness.
                    let mut empty = false;
                    for decision in facts {
                        checkpoint(context, observation_work(decision, locus)).map_err(
                            |cause| {
                                request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                            },
                        )?;
                        let Decision::Application { read_scope, fact } = decision else {
                            continue;
                        };
                        if !scope_matches(read_scope.as_ref(), locus) {
                            continue;
                        }
                        // The field footprint is the existing actor correlation;
                        // its traversal is paid even when it finds no shared record.
                        checkpoint(context, field_mutations.records().count() as u64).map_err(
                            |cause| {
                                request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                            },
                        )?;
                        empty = match fact.as_ref() {
                            Fact::Relation {
                                relation_kind,
                                from,
                                to,
                                matching_relations,
                            } => {
                                *relation_kind == kind
                                    && matching_relations.is_empty()
                                    && field_mutations.records().any(|record| {
                                        *record == RecordRef::Entity(*from)
                                            || *record == RecordRef::Entity(*to)
                                    })
                            }
                            Fact::Adjacency {
                                relation_kind,
                                anchor,
                                relations,
                                ..
                            } => {
                                *relation_kind == kind
                                    && relations.is_empty()
                                    && field_mutations
                                        .records()
                                        .any(|record| *record == RecordRef::Entity(*anchor))
                            }
                            _ => false,
                        };
                        if empty {
                            break;
                        }
                    }
                    if !empty {
                        return Err(denied(
                            Cause::MissingMutation,
                            retained.target_record().cloned().into_iter().collect(),
                        ));
                    }
                }
            }
            Ok::<(), WorthQueryProviderSessionFailure>(())
        })
        .map_err(request_stop)?
        .0?;
    Ok(retained.with_relation_seal(
        WorthQueryRetainedRelationPreImageSeal {
            rows,
            loci: demand.relations().to_vec(),
            footprint,
            memory,
        },
        total,
    ))
}
fn scope_matches(
    scope: &WorthQueryOperationGraphReadScope,
    locus: &InstalledPreImageRelationLocus,
) -> bool {
    matches!(scope, WorthQueryOperationGraphReadScope::Relation(scope) if scope.relation() == locus.relation() && scope.from() == locus.from() && scope.to() == locus.to())
}
fn observes(
    fact: &Decision,
    locus: &InstalledPreImageRelationLocus,
    kind: worth_relational::facade::identity::KindId,
    id: worth_relational::facade::identity::RelationId,
    ends: (
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::identity::EntityId,
    ),
    absent: bool,
) -> bool {
    let Decision::Application { read_scope, fact } = fact else {
        return false;
    };
    if !scope_matches(read_scope.as_ref(), locus) {
        return false;
    }
    match fact.as_ref() {
        Fact::Relation {
            relation_kind,
            from,
            to,
            matching_relations,
        } => {
            *relation_kind == kind
                && (*from, *to) == ends
                && if absent {
                    matching_relations.is_empty()
                } else {
                    matching_relations.contains(&id)
                }
        }
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            relations,
            ..
        } => {
            if *relation_kind != kind
                || *anchor
                    != if *direction == Direction::Outgoing {
                        ends.0
                    } else {
                        ends.1
                    }
            {
                return false;
            }
            if absent {
                !relations.iter().any(|edge| (edge.from, edge.to) == ends)
            } else {
                relations
                    .iter()
                    .any(|edge| edge.relation_id == id && (edge.from, edge.to) == ends)
            }
        }
        _ => false,
    }
}
pub(super) fn denied(kind: Cause, records: Vec<RecordRef>) -> WorthQueryProviderSessionFailure {
    provider_failure(
        WorthQueryProviderSessionProtocolStage::Commit,
        "recorded inverse relation pre-image refused",
    )
    .with_relation_preimage_failure(Failure::new(kind, records))
}
fn request_stop(cause: worth_execution::WorkCeilingDenial) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::checked_read_stopped(crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause), WorthQueryProviderSessionProtocolStage::Commit, "relation pre-image request stopped")
}

fn observation_work(decision: &Decision, locus: &InstalledPreImageRelationLocus) -> u64 {
    let Decision::Application { read_scope, fact } = decision else {
        return 1;
    };
    let WorthQueryOperationGraphReadScope::Relation(scope) = read_scope.as_ref() else {
        return 1;
    };
    let mut work = 1u64;
    for length in [
        scope.relation().len(),
        scope.from().len(),
        scope.to().len(),
        locus.relation().len(),
        locus.from().len(),
        locus.to().len(),
    ] {
        work = work.saturating_add(length as u64);
    }
    work.saturating_add(match fact.as_ref() {
        Fact::Relation {
            matching_relations, ..
        } => matching_relations.len() as u64,
        Fact::Adjacency { relations, .. } => relations.len() as u64,
        _ => 0,
    })
}
