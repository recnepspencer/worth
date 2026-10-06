use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as DecisionDirection,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::runtime::{
    AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection as NativeDirection,
};

// A relation-pair read has no declared traversal bound, so use a fixed small
// ceiling, never a limit inferred from its matched pair count. A broader
// outgoing revision may over-invalidate but covers pair presence and ABA.
const MAXIMUM_PAIR_REBASE_WORK: usize = 64;

#[cfg(test)]
pub(super) fn rebase_decision_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: Fact,
    admitted_work: usize,
) -> Fact {
    prepare_decision_adjacency(runtime, snapshot, &fact, admitted_work).unwrap_or(fact)
}

pub(super) fn prepare_decision_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: &Fact,
    admitted_work: usize,
) -> Option<Fact> {
    if admitted_work == 0 {
        return None;
    }
    let (relation_kind, anchor, direction, limit) = match fact {
        Fact::Relation {
            relation_kind,
            from,
            ..
        } => (
            *relation_kind,
            *from,
            NativeDirection::Outgoing,
            MAXIMUM_PAIR_REBASE_WORK.min(admitted_work),
        ),
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            maximum_work_units,
            ..
        } => {
            let native_direction = match direction {
                DecisionDirection::Outgoing => NativeDirection::Outgoing,
                DecisionDirection::Incoming => NativeDirection::Incoming,
            };
            (
                *relation_kind,
                *anchor,
                native_direction,
                (*maximum_work_units).min(admitted_work),
            )
        }
        _ => return None,
    };
    let view = runtime.read_truth().project_snapshot(snapshot)?;
    let native_revision =
        match view.bounded_adjacency_structural_revision(anchor, relation_kind, direction, limit) {
            Ok(revision) => revision.revision(),
            // A zero recorded limit, or a missing anchor or basis, leaves the
            // decision fact without a native revision.
            Err(
                AdjacencyStructuralRevisionDenial::WorkBudgetExceeded
                | AdjacencyStructuralRevisionDenial::AnchorUnavailable
                | AdjacencyStructuralRevisionDenial::BasisUnavailable,
            ) => return None,
        };
    let endpoints = match fact {
        Fact::Relation { to, .. } => vec![*to],
        Fact::Adjacency {
            direction,
            relations,
            ..
        } => relations
            .iter()
            .map(|relation| match direction {
                DecisionDirection::Outgoing => relation.to,
                DecisionDirection::Incoming => relation.from,
            })
            .collect(),
        _ => unreachable!("decision adjacency was classified above"),
    };
    Some(Fact::SourceAdjacencyRevision {
        relation_kind,
        anchor,
        direction,
        native_revision,
        comparison_work_limit: limit,
        endpoints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope, AccountOwner, PrincipalIdentityField,
    };
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

    fn admission(work: u64) -> crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission{
        crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: work,
                maximum_preparation_bytes: 1024 * 1024,
            },
        )
    }

    #[test]
    fn pair_rebase_requires_admitted_native_work_and_never_retains_partial_facts() {
        let world = installed_authorization_world(true);
        let graph = world.application.runtime.primary_graph().unwrap();
        let relation_kind = graph
            .layout
            .relation(AccountOwner::reference().name())
            .unwrap()
            .kind;
        let selected = world.selected_product();
        let principal = selected
            .resolve_entity(
                PrincipalIdentityField::reference(),
                1,
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let fact = Fact::Relation {
            relation_kind,
            from: principal.entity_id(),
            to: principal.entity_id(),
            matching_relations: Vec::new(),
        };
        graph.integration_handle().with_runtime(|runtime| {
            let snapshot = selected.application_basis().snapshot_handle();
            let mut zero_work = admission(0);
            assert!(matches!(
                rebase_decision_adjacency(runtime, snapshot, fact.clone(), 64),
                Fact::SourceAdjacencyRevision {
                    comparison_work_limit: 64,
                    native_revision: Some(_),
                    ..
                }
            ));
            assert!(
                matches!(
                    super::super::rebase(
                        runtime,
                        snapshot,
                        super::super::PreparedSourceFactRebase::admit(vec![fact]).unwrap(),
                        &std::collections::BTreeSet::new(),
                        true,
                        0,
                        &mut zero_work
                    ),
                    super::super::RebasedSourceFacts::VerificationRequired { .. }
                ),
                "failed relation revision acquisition marks the entire output non-reusable"
            );
            let stale_revision = Fact::SourceAdjacencyRevision {
                relation_kind,
                anchor: principal.entity_id(),
                direction: NativeDirection::Outgoing,
                native_revision: None,
                comparison_work_limit: 0,
                endpoints: Vec::new(),
            };
            let mut producer_work = admission(64);
            assert!(matches!(
                super::super::rebase(
                    runtime,
                    snapshot,
                    super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()])
                        .unwrap(),
                    &std::collections::BTreeSet::new(),
                    true,
                    64,
                    &mut producer_work,
                ),
                super::super::RebasedSourceFacts::VerificationRequired { .. }
            ));
            let mut ordinary_work = admission(64);
            match super::super::rebase(
                runtime,
                snapshot,
                super::super::PreparedSourceFactRebase::admit(vec![stale_revision.clone()])
                    .unwrap(),
                &std::collections::BTreeSet::new(),
                false,
                64,
                &mut ordinary_work,
            ) {
                super::super::RebasedSourceFacts::Exact(facts) => {
                    assert_eq!(facts.as_ref(), &[stale_revision])
                }
                other => {
                    panic!("non-output evidence must retain its comparison posture: {other:?}")
                }
            }
        });
    }
}
