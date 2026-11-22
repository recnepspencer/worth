//! Existing-entity writes require the exact completed decision field address.
use super::WorthQueryApplicationEffectProgramBuilder;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(super) fn require_field_observation(
        &self,
        entity_id: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        super::core::require_field_observation(
            &self.read_set.effect_routes,
            &self.read_set.facts,
            entity_id,
            locator,
        )
    }
}
