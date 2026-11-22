//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Relation, Entity> WorthQueryInvariantProjectionWalk<Schema, Relation, Entity> {
    #[doc(hidden)]
    pub fn issued_at_for_test(
        &self,
        origin: &crate::domain_computation::primary_graph::projection_read_origin::ProjectionReadOriginRef<'_>,
        cookie: u64,
    ) -> bool {
        Self::issued_at(self, origin, cookie)
    }
}
