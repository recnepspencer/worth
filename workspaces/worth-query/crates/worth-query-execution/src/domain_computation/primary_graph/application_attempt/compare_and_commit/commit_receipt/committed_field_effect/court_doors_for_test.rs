//! Sealed test doors over this module's private members.
use super::*;

pub trait CommittedFieldEffectDoorsOps {
    #[doc(hidden)]
    fn field_effect_for_test(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop>;
}

impl CommittedFieldEffectDoorsOps for WorthQueryApplicationCommittedChanges {
    fn field_effect_for_test(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop> {
        <Self as CommittedFieldEffect>::field_effect(self, context, entity, coordinate)
    }
}
