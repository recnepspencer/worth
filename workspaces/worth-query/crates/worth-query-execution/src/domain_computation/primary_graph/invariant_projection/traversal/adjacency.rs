//! All projection adjacency callers acquire physical records through this owner.
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::BoundedAdjacencyTruthRead,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn read_adjacency(
        &mut self,
        subject: &str,
        member: EntityId,
        kind: KindId,
        direction: RelationDirection,
    ) -> Result<BoundedAdjacencyTruthRead, WorthQueryInvariantProjectionTraversalDenial> {
        self.native_read()
            .read_adjacency(subject, member, kind, direction)
    }
}

mod court_doors_for_test;
