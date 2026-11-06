//! Complete structural reads through the existing snapshot projection authority.

use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::ApplicationRelationRef;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use super::WorthQueryApplicationInvariantProjectionReader;
use super::WorthQueryInvariantEntityIdentity;
use super::WorthQueryInvariantProjectionTraversalDenial as Denial;

mod result;

#[cfg(test)]
mod tests;
pub use result::{
    WorthQueryInvariantProjectionWalk, WorthQueryInvariantProjectionWalkIter,
    WorthQueryInvariantProjectionWalkStep,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Complete reachability, including the root at distance zero.
    pub fn walk<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>, Denial> {
        self.closure(relation, direction, std::slice::from_ref(root))
    }

    /// One global multi-root walk, ordered by minimum distance then identity.
    pub fn closure<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>, Denial> {
        let observation = self.native_read().closure(
            relation.name(),
            relation.from(),
            direction,
            &super::native_read::TypedRoots(roots),
        )?;
        Ok(WorthQueryInvariantProjectionWalk::new(
            observation,
            roots.first(),
            self.authority_identity,
        ))
    }

    /// Forward reflexive reachability. Both identities must belong to this reader.
    pub fn reaches<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, Denial> {
        self.native_read().reaches(
            relation.name(),
            relation.from(),
            relation.to(),
            from.native_identity(),
            to.native_identity(),
        )
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn walk_stop(
        &mut self,
        cause: Cause,
        subject: &str,
    ) -> Denial {
        self.native_read().walk_stop(cause, subject)
    }
}
