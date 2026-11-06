//! Interpret one canonical occurrence through its actual installed Native coordinate.
use crate::domain_computation::graph_layout::InstalledFieldCoordinate;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::committed_changes::WorthQueryApplicationCommittedChanges;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::publication::RecordStructuralChange;
use worth_relational::facade::transactions::RecordRef;
use worth_relational::facade::publication::PublishedAuthoritativeFieldEffect as Effect;

/// The attempt's reading of one committed field through its installed Native
/// coordinate.
pub(in crate::domain_computation::primary_graph) trait CommittedFieldEffect {
    /// O(record patches + matching patch grammar), diagnostic only and allocation-free.
    /// Full-GEN identity and one original patch are required. Retirement supplies no field.
    fn field_effect(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop>;
}

impl CommittedFieldEffect for WorthQueryApplicationCommittedChanges {
    fn field_effect(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<Effect, worth_execution::MapKernelStop> {
        let mut result = Effect::Unchanged;
        let mut seen = false;
        for patch in self.record_patches() {
            crate::domain_computation::vocabulary::primary_graph::admitted_capture::checkpoint(
                context, 1,
            )?;
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
                    crate::domain_computation::vocabulary::primary_graph::admitted_capture::checkpoint(
                        context, work,
                    )
                },
            )?;
        }
        Ok(result)
    }
}

pub(crate) mod court_doors_for_test;
