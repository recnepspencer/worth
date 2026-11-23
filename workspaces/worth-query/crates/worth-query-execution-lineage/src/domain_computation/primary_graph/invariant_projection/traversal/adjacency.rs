//! All projection adjacency callers acquire physical records through this owner.
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationReadRecord,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(crate) fn read_adjacency(
        &mut self,
        subject: &str,
        member: EntityId,
        kind: KindId,
        direction: RelationDirection,
    ) -> Result<Vec<RelationReadRecord>, WorthQueryInvariantProjectionTraversalDenial> {
        self.native_read()
            .read_adjacency(subject, member, kind, direction)
    }
}

mod court_doors_for_test;
