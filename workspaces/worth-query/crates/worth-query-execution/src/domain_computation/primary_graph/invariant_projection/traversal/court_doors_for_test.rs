//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn endpoint_for_test<Entity>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: worth_relational::facade::identity::EntityId,
        expected_kind: worth_relational::facade::identity::KindId,
        entity: &str,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        Self::endpoint::<Entity>(self, execution, entity_id, expected_kind, entity)
    }
}
