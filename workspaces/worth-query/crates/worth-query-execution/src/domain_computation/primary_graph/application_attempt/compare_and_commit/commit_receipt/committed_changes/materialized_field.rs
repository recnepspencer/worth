//! An immutable original envelope and full-generation patch seal a Native value position.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::checkpoint, schema_layout::InstalledFieldCoordinate,
};
use worth_execution::{MapKernelContext, MapKernelStop};
use worth_relational::facade::publication::{
    PublishedAuthoritativeFieldEffect as Effect, PublishedAuthoritativeFieldValueLocation,
};

pub(in crate::domain_computation::primary_graph) struct MaterializedCommittedField {
    original: WorthQueryApplicationCommittedChanges,
    patch: Option<usize>,
    effect: Effect,
    value: Option<PublishedAuthoritativeFieldValueLocation>,
}
impl MaterializedCommittedField {
    pub(in crate::domain_computation::primary_graph) fn effect(&self) -> Effect {
        self.effect
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> Option<&AspectValue> {
        self.original
            .envelope
            .patch
            .authoritative_record_patches
            .get(self.patch?)?
            .authoritative_patch
            .field_value_at(self.value?)
    }
}
impl WorthQueryApplicationCommittedChanges {
    pub(in crate::domain_computation::primary_graph) fn materialize_field(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        entity: EntityId,
        coordinate: InstalledFieldCoordinate<'_>,
    ) -> Result<MaterializedCommittedField, MapKernelStop> {
        let mut selected = None;
        for (index, patch) in self
            .envelope
            .patch
            .authoritative_record_patches
            .iter()
            .enumerate()
        {
            checkpoint(context, 1)?;
            if patch.target != RecordRef::Entity(entity) {
                continue;
            }
            if selected.replace(index).is_some() {
                return Ok(MaterializedCommittedField {
                    original: self.clone(),
                    patch: None,
                    effect: Effect::Ambiguous,
                    value: None,
                });
            }
        }
        let (effect, value) = match selected {
            None => (Effect::Unchanged, None),
            Some(index) => {
                let patch = &self.envelope.patch.authoritative_record_patches[index];
                if !matches!(
                    patch.structural_change,
                    RecordStructuralChange::Created | RecordStructuralChange::Updated
                ) {
                    (Effect::Unsupported, None)
                } else {
                    patch.authoritative_patch.field_effect_with_value_location(
                        coordinate.native().contract(),
                        coordinate.native().binding(),
                        coordinate.field(),
                        |work| checkpoint(context, work),
                    )?
                }
            }
        };
        Ok(MaterializedCommittedField {
            original: self.clone(),
            patch: selected,
            effect,
            value,
        })
    }
}
