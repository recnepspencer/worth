//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommittedChanges {
    #[doc(hidden)]
    pub fn field_effect_for_test(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop> {
        Self::field_effect(self, context, entity, coordinate)
    }
}
