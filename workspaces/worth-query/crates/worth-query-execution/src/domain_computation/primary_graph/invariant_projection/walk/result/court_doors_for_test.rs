//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Relation, Entity> WorthQueryInvariantProjectionWalk<Schema, Relation, Entity> {
    #[doc(hidden)]
    pub fn issued_at_for_test(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        cookie: u64,
    ) -> bool {
        Self::issued_at(self, snapshot, cookie)
    }
}
