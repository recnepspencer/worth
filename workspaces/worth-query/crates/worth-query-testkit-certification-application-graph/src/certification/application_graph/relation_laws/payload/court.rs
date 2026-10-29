//! The authored handler publishes real relation payload; failed writers leave it unchanged.
pub use super::super::{installation::*, outcome::committed};
pub use super::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope;
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome,
    WorthQueryApplicationRequestMutationDenial as RequestDenial,
};
pub use worth_relational::facade::identity::RelationId;

pub fn payloads(world: &World<Payload>) -> Vec<(u64, u64, RelationId, u64)> {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let mut values = Vec::new();
                        for source in 0..world.nodes {
                            let source_identity =
                                reader.resolve_entity(NodeKey::reference(), source).unwrap();
                            for edge in reader
                                .relations_from(Precedes::reference(), &source_identity)
                                .unwrap()
                            {
                                let target = reader.field(edge.to(), NodeKey::reference()).unwrap();
                                let weight = reader
                                    .relation_field(&edge, EdgeWeight::reference())
                                    .expect("actual native initialized payload");
                                values.push((source, target, edge.relation_id(), weight));
                            }
                        }
                        values.sort();
                        values
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (values, snapshot, _) = completed.into_parts();
    drop(snapshot);
    values
}
