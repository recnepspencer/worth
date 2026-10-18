//! Own-effect normalization creates a separate funded body and content root.
use super::super::super::{
    effect_program::WorthQueryApplicationRealizedEffect as Effect,
    WorthQueryApplicationObservedFact as Fact,
};
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;

impl FactContent {
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
                    count.checked_add(self.0.fields.ordinals(entity, locator).len())
                });
            });
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
                for &ordinal in self.0.fields.ordinals(entity, locator) {
                    // Preserve the original first matching effect rule.
                    writes.entry(ordinal).or_insert(value);
                }
            });
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
                .checked_add(value.map_or(0, AspectValue::owned_allocation_capacity_bytes))
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or(StoreDenial::Representability)?;
            let held = control.policy().reserve_memory(quote);
            control.check_live()?;
            let _held = held?;
            let next = match value {
                Some(value) => Fact::Field {
                    entity_id,
                    kind,
                    locator: locator.clone(),
                    value: value.clone(),
                },
                None => Fact::AbsentField {
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
        Option<&'a AspectValue>,
    ),
) {
    match effect {
        Effect::UpdateEntity {
            entity_id, fields, ..
        } => {
            for (locator, value) in fields {
                write(*entity_id, locator, Some(value));
            }
        }
        Effect::PatchOptionalEntityFields {
            entity_id, fields, ..
        } => {
            for (locator, value) in fields {
                write(*entity_id, locator, value.value.as_ref());
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
