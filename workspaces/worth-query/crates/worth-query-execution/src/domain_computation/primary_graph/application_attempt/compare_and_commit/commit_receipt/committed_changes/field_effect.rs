//! Interpret one canonical occurrence through its actual installed Native coordinate.
use super::*;
use crate::domain_computation::primary_graph::schema_layout::InstalledFieldCoordinate;
use worth_relational::facade::publication::PublishedAuthoritativeFieldEffect as Effect;

impl WorthQueryApplicationCommittedChanges {
    /// O(record patches + matching patch grammar), diagnostic only and allocation-free.
    /// Full-GEN identity and one original patch are required. Retirement supplies no field.
    pub(in crate::domain_computation::primary_graph) fn field_effect(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop> {
        let mut result = Effect::Unchanged;
        let mut seen = false;
        for patch in &self.envelope.patch.authoritative_record_patches {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)?;
            if patch.target != RecordRef::Entity(entity) {
                continue;
            }
            if seen {
                return Ok(Effect::Ambiguous);
            }
            seen = true;
            if !matches!(
                patch.structural_change,
                RecordStructuralChange::Created | RecordStructuralChange::Updated
            ) {
                return Ok(Effect::Unsupported);
            }
            result = patch.authoritative_patch.field_effect(
                coordinate.native().contract(),
                coordinate.native().binding(),
                coordinate.field(),
                |work| {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, work,
                    )
                },
            )?;
        }
        Ok(result)
    }
}
