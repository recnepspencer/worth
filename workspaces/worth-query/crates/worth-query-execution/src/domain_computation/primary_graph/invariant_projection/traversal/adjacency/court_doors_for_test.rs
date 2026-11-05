//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    #[doc(hidden)]
    pub fn read_adjacency_for_test(
        &mut self,
        subject: &str,
        member: EntityId,
        kind: KindId,
        direction: RelationDirection,
    ) -> Result<BoundedAdjacencyTruthRead, WorthQueryInvariantProjectionTraversalDenial> {
        Self::read_adjacency(self, subject, member, kind, direction)
    }
}
