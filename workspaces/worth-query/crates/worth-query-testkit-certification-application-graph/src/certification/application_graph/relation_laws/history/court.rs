//! Authored actions, receipt coordinates and independent committed identities form the oracle.
pub use super::super::installation::key;
pub use super::super::{
    creation::{Create, CreateNodeInput},
    feeding::FeedsChange,
    installation::*,
    outcome::committed,
    read,
};
pub use super::access::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::marker::PhantomData;
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    application_invariants::{EntityId, RelationId},
    runtime::ExecutionAllocationPolicy,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Event {
    pub commit: u64,
    pub entity: Option<EntityId>,
    pub relation: Option<RelationId>,
    pub structure: RecordStructuralChange,
}
pub fn events(page: &WorthQueryApplicationHistoryPage) -> Vec<Event> {
    assert!(page.source_visits() <= 1);
    assert!(page.entries().count() <= 1);
    assert!(page.newly_owned_bytes() <= limits().maximum_owned_bytes.get());
    page.entries()
        .map(|entry| {
            assert!(entry.changed_aspects().count() <= 1);
            if entry.structural_change() == RecordStructuralChange::Created {
                assert_eq!(entry.changed_aspects().count(), 1);
            }
            Event {
                commit: entry.commit_id().0,
                entity: entry.entity_id(),
                relation: entry.relation_id(),
                structure: entry.structural_change(),
            }
        })
        .collect()
}
pub fn receipt(outcome: Outcome<GraphDenial, Edited>) -> WorthQueryApplicationCommitReceipt {
    match outcome {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("authored transaction must commit: {other:?}"),
    }
}

pub fn physical(world: &World<Payload>) -> Vec<(u64, RelationId, u64)> {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let source = reader.resolve_entity(NodeKey::reference(), 0).unwrap();
                        let mut ids = reader
                            .relations_from(Precedes::reference(), &source)
                            .unwrap()
                            .into_iter()
                            .map(|edge| {
                                (
                                    reader.field(edge.to(), NodeKey::reference()).unwrap(),
                                    edge.relation_id(),
                                    reader
                                        .relation_field(&edge, EdgeWeight::reference())
                                        .unwrap(),
                                )
                            })
                            .collect::<Vec<_>>();
                        ids.sort();
                        ids
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (ids, snapshot, _) = completed.into_parts();
    drop(snapshot);
    ids
}
