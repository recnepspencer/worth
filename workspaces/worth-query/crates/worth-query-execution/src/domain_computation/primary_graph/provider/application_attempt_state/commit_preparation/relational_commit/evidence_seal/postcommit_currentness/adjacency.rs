use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as DecisionDirection,
    WorthQueryApplicationObservedFact as Fact,
};
use worth_relational::facade::runtime::RelationalAdjacencyDirection as NativeDirection;

// A relation-pair read has no declared traversal bound, so use a fixed small
// ceiling, never a limit inferred from its matched pair count. A broader
// outgoing revision may over-invalidate but covers pair presence and ABA.
const MAXIMUM_PAIR_REBASE_WORK: usize = 64;

pub(super) fn rebase_decision_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    fact: Fact,
    admitted_work: usize,
) -> Fact {
    if admitted_work == 0 {
        return fact;
    }
    let (relation_kind, anchor, direction, limit, endpoints) = match &fact {
        Fact::Relation {
            relation_kind,
            from,
            to,
            ..
        } => (
            *relation_kind,
            *from,
            NativeDirection::Outgoing,
            MAXIMUM_PAIR_REBASE_WORK.min(admitted_work),
            vec![*to],
        ),
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            maximum_work_units,
            relations,
        } => {
            let native_direction = match direction {
                DecisionDirection::Outgoing => NativeDirection::Outgoing,
                DecisionDirection::Incoming => NativeDirection::Incoming,
            };
            let endpoints = relations
                .iter()
                .map(|relation| match direction {
                    DecisionDirection::Outgoing => relation.to,
                    DecisionDirection::Incoming => relation.from,
                })
                .collect();
            (
                *relation_kind,
                *anchor,
                native_direction,
                (*maximum_work_units).min(admitted_work),
                endpoints,
            )
        }
        _ => return fact,
    };
    let Some(native_revision) = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .and_then(|view| {
            view.bounded_adjacency_structural_revision(anchor, relation_kind, direction, limit)
                .ok()
        })
        .map(|revision| revision.revision())
    else {
        return fact;
    };
    Fact::SourceAdjacencyRevision {
        relation_kind,
        anchor,
        direction,
        native_revision,
        comparison_work_limit: limit,
        endpoints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope, AccountOwner, PrincipalIdentityField,
    };
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

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
            assert!(matches!(
                rebase_decision_adjacency(runtime, snapshot, fact.clone(), 64),
                Fact::SourceAdjacencyRevision {
                    comparison_work_limit: 64,
                    native_revision: Some(_),
                    ..
                }
            ));
            assert_eq!(
                super::super::rebase(runtime, snapshot, vec![fact], true, 0).len(),
                0,
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
            assert!(super::super::rebase(
                runtime,
                snapshot,
                vec![stale_revision.clone()],
                true,
                64,
            )
            .is_empty());
            assert_eq!(
                super::super::rebase(runtime, snapshot, vec![stale_revision.clone()], false, 64)
                    .as_ref(),
                &[stale_revision],
                "non-output evidence retains its original comparison posture"
            );
        });
    }
}
