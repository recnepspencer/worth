//! Own-effect normalization creates a separate funded body and content root.
use super::super::super::{
    effect_program::WorthQueryApplicationRealizedEffect as Effect,
    WorthQueryApplicationObservedFact as Fact,
};
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
#[derive(Clone, Copy)]
enum FieldPostcondition<'a> {
    Present(&'a AspectValue),
    FieldAbsent,
    AspectAbsent,
    ParentAbsent,
}
impl FieldPostcondition<'_> {
    fn value_capacity(self) -> usize {
        match self {
            Self::Present(value) => value.owned_allocation_capacity_bytes(),
            Self::FieldAbsent | Self::AspectAbsent | Self::ParentAbsent => 0,
        }
    }
}

impl FactContent {
    // Whole-aspect absence must reach the committed native presence probe,
    // including ordinary handler reads and clears that leave no aspect.
    fn normalizable_field_ordinals<'a>(
        &'a self,
        entity: worth_relational::facade::identity::EntityId,
        locator: &worth_foundational::facade::AspectFieldLocator,
    ) -> impl Iterator<Item = usize> + 'a {
        self.0
            .fields
            .ordinals(entity, locator)
            .iter()
            .copied()
            .filter(|ordinal| !matches!(self.0.body[*ordinal], Fact::OptionalAspectAbsent { .. }))
    }

    // Only the new parent-removal path visits explicit presence observations.
    // No retained index or history is added; every visited fact polls the request.
    fn each_removed_presence(
        &self,
        effect: &Effect,
        control: StorageControl<'_, '_>,
        mut visit: impl FnMut(usize),
    ) -> Result<(), StoreDenial> {
        let Effect::RemoveAspect {
            entity_id: target,
            contract,
            ..
        } = effect
        else {
            return Ok(());
        };
        for (ordinal, fact) in self.0.body.iter().enumerate() {
            control.check_live()?;
            if matches!(fact, Fact::AspectPresence { entity_id, aspect, .. } if entity_id == target && aspect == contract.key())
            {
                visit(ordinal);
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn postcondition(
        &self,
        effects: &[Effect],
        computation_ordinals: &[usize],
        control: StorageControl<'_, '_>,
    ) -> Result<(FactPayload, Self, Vec<usize>, Work), StoreDenial> {
        let mut count = Some(0usize);
        for effect in effects {
            each_write(effect, |entity, locator, _| {
                count = count.and_then(|count| {
                    count.checked_add(self.normalizable_field_ordinals(entity, locator).count())
                });
            });
        }
        for effect in effects {
            self.each_removed_presence(effect, control, |_| {
                count = count.and_then(|n| n.checked_add(1));
            })?;
        }
        let quote = count
            .and_then(|count| u64::try_from(count).ok()?.checked_mul(2_048))
            .ok_or(StoreDenial::Representability)?;
        let funding = control.policy().reserve_memory(quote);
        control.check_live()?;
        let _funding = funding?;
        let mut writes = BTreeMap::new();
        for effect in effects {
            control.check_live()?;
            each_write(effect, |entity, locator, value| {
                for ordinal in self.normalizable_field_ordinals(entity, locator) {
                    // Preserve the original first matching effect rule.
                    writes.entry(ordinal).or_insert(value);
                }
            });
        }
        for effect in effects {
            self.each_removed_presence(effect, control, |ordinal| {
                writes
                    .entry(ordinal)
                    .or_insert(FieldPostcondition::ParentAbsent);
            })?;
        }
        if writes.is_empty() {
            return Ok((self.0.body.clone(), self.clone(), Vec::new(), Work::zero()));
        }
        let changed: im::OrdSet<_> = writes.keys().copied().collect();
        let mut patch = self.0.body.prepare_native_patch(&changed, control)?;
        let mut moved = Vec::new();
        for (ordinal, value) in writes {
            control.check_live()?;
            let original = &self.0.body[ordinal];
            if let (
                FieldPostcondition::ParentAbsent,
                Fact::AspectPresence {
                    entity_id,
                    kind,
                    aspect,
                    ..
                },
            ) = (value, original)
            {
                let held = control
                    .policy()
                    .reserve_memory(original.additional_charged_bytes());
                control.check_live()?;
                let _held = held?;
                let next = Fact::AspectPresence {
                    entity_id: *entity_id,
                    kind: *kind,
                    aspect: aspect.clone(),
                    present: false,
                };
                if original != &next && computation_ordinals.binary_search(&ordinal).is_ok() {
                    moved.push(ordinal);
                }
                patch.replace(ordinal, &next, control)?;
                continue;
            }
            let (entity_id, kind, locator) = match original {
                Fact::Field {
                    entity_id,
                    kind,
                    locator,
                    ..
                }
                | Fact::AbsentField {
                    entity_id,
                    kind,
                    locator,
                } => (*entity_id, *kind, locator),
                _ => return Err(StoreDenial::ConflictingBody),
            };
            // The borrowed effect owns its value. This NEW temporary clone has
            // its own admission; patch.replace funds the surviving clone.
            let quote = locator
                .owned_allocation_capacity_bytes()
                .checked_add(value.value_capacity())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(StoreDenial::Representability)?;
            let held = control.policy().reserve_memory(quote);
            control.check_live()?;
            let _held = held?;
            let next = match value {
                FieldPostcondition::Present(value) => Fact::Field {
                    entity_id,
                    kind,
                    locator: locator.clone(),
                    value: value.clone(),
                },
                FieldPostcondition::AspectAbsent => Fact::OptionalAspectAbsent {
                    entity_id,
                    kind,
                    locator: locator.clone(),
                },
                FieldPostcondition::ParentAbsent => return Err(StoreDenial::ConflictingBody),
                FieldPostcondition::FieldAbsent => Fact::AbsentField {
                    entity_id,
                    kind,
                    locator: locator.clone(),
                },
            };
            if original != &next && computation_ordinals.binary_search(&ordinal).is_ok() {
                moved.push(ordinal);
            }
            patch.replace(ordinal, &next, control)?;
        }
        let body = patch.seal(control)?;
        let (content, work) = self.refresh(body.clone(), changed, control)?;
        Ok((body, content, moved, work))
    }
}

fn each_write<'a>(
    effect: &'a Effect,
    mut write: impl FnMut(
        worth_relational::facade::identity::EntityId,
        &worth_foundational::facade::AspectFieldLocator,
        FieldPostcondition<'a>,
    ),
) {
    match effect {
        Effect::UpdateEntity {
            entity_id, fields, ..
        } => {
            for (locator, value) in fields {
                write(*entity_id, locator, FieldPostcondition::Present(value));
            }
        }
        Effect::PatchOptionalEntityFields {
            entity_id, fields, ..
        } => {
            for (locator, value) in fields {
                write(
                    *entity_id,
                    locator,
                    value
                        .value
                        .as_ref()
                        .map_or(FieldPostcondition::FieldAbsent, FieldPostcondition::Present),
                );
            }
        }
        Effect::RemoveAspect {
            entity_id, fields, ..
        } => {
            for locator in fields {
                write(*entity_id, locator, FieldPostcondition::AspectAbsent);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
