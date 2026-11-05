//! Repeated case writes replace only their own pending sibling-clear slots.
use super::fragment::{prepare_fields, restore};
use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationEffectProgramBuilder as Builder,
    WorthQueryApplicationRealizedEffect as Effect,
};
use crate::domain_computation::primary_graph::fragment_view::{ErasedCandidate, FragmentDenial};
use std::collections::BTreeMap;
use worth_foundational::facade::{
    AbsenceLaw, AspectKey, AspectShape, AspectValue, PortableAspectContractBasis,
};
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, GroupDefinition,
};
use worth_query_installation::facade::ApplicationOperationProgramTarget as Target;
use worth_relational::facade::{identity::EntityId, transactions::EntityReference};

#[cfg(test)]
mod tests;

pub(super) type CaseTransitions = BTreeMap<(EntityId, std::any::TypeId), Vec<usize>>;

impl<S, O, I, Scope> Builder<S, O, I, Scope> {
    pub(super) fn write_case_record(
        &mut self,
        target: &ErasedCandidate,
        selected: &AspectDefinition,
        group: &GroupDefinition,
        cases: &[&AspectDefinition],
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial> {
        let native = restore::<S>(target);
        self.validate_target(&native, selected.entity())?;
        let prepared = prepare_fields(selected, fields)?;
        let siblings: Vec<_> = cases
            .iter()
            .copied()
            .filter(|case| case.name() != selected.name())
            .collect();
        for sibling in &siblings {
            self.admit_program_target(&Target::RemoveAspect {
                entity: sibling.entity().to_owned(),
                aspect: sibling.name().to_owned(),
            })?;
        }
        if let EntityReference::Existing(id) = native.reference {
            let key = (id, group.marker());
            let old_slots = self.case_transitions.get(&key).cloned();
            let owned = old_slots.as_deref().unwrap_or(&[]);
            check_case_slots(&self.effects, id, selected.entity(), cases, owned)?;
            self.preflight_record(&native, selected, owned)?;
            let removals = siblings
                .iter()
                .map(|sibling| self.prepare_case_clear(id, sibling))
                .collect::<Result<Vec<_>, _>>()?;
            let mut fresh_slots = Vec::new();
            if let Some(slots) = &old_slots {
                if slots.len() != removals.len() {
                    return Err(Denial::new(Kind::ConflictingEffectStep, selected.path()).into());
                }
            } else {
                fresh_slots
                    .try_reserve(removals.len())
                    .map_err(|_| Denial::new(Kind::AllocationDenied, selected.path()))?;
                for offset in 0..removals.len() {
                    fresh_slots.push(
                        self.effects
                            .len()
                            .checked_add(offset)
                            .ok_or_else(|| Denial::new(Kind::AllocationDenied, selected.path()))?,
                    );
                }
            }
            self.reserve_effects(if old_slots.is_some() {
                1
            } else {
                siblings.len().saturating_add(1)
            })?;
            self.clear_pending_case_fields(id, cases);
            if let Some(slots) = old_slots {
                for (slot, removal) in slots.into_iter().zip(removals) {
                    self.effects[slot] = removal;
                }
            } else {
                self.effects.extend(removals);
                self.case_transitions.insert(key, fresh_slots);
            }
            for (locus, value) in prepared {
                self.write_body_input_field(&native, locus, value)?;
            }
        } else {
            self.preflight_record(&native, selected, &[])?;
            let index = native
                .created_effect
                .ok_or_else(|| Denial::new(Kind::ForeignEffectTarget, selected.path()))?;
            let Some(Effect::CreateEntity { fields, .. }) = self.effects.get_mut(index) else {
                return Err(Denial::new(Kind::ForeignEffectTarget, selected.path()).into());
            };
            fields.retain(|field, _| !in_cases(field, cases));
            for (locus, value) in prepared {
                self.initialize_body_field(&native, locus, std::borrow::Cow::Owned(value))?;
            }
        }
        Ok(())
    }

    fn prepare_case_clear(
        &self,
        id: EntityId,
        aspect: &AspectDefinition,
    ) -> Result<Effect, Denial> {
        let key = AspectKey::new(aspect.name())
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, aspect.path()))?;
        let contract = self
            .layout
            .aspect_contract(aspect.entity(), &key)
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, aspect.path()))?;
        if contract.absence() != AbsenceLaw::Optional {
            return Err(Denial::new(Kind::UndeclaredEffect, aspect.path()));
        }
        let AspectShape::Struct(shape) = contract.shape() else {
            return Err(Denial::new(Kind::UndeclaredEffect, aspect.path()));
        };
        if shape.fields().is_empty() {
            return Err(Denial::new(Kind::UndeclaredEffect, aspect.path()));
        }
        let mut fields = Vec::new();
        fields
            .try_reserve(shape.fields().len())
            .map_err(|_| Denial::new(Kind::AllocationDenied, aspect.path()))?;
        for field in shape.fields() {
            let locator =
                self.field_locator(aspect.entity(), aspect.name(), field.key().as_str())?;
            self.require_field_observation(id, &locator)?;
            fields.push(locator);
        }
        Ok(Effect::RemoveAspect {
            entity: aspect.entity().to_owned(),
            entity_id: id,
            contract: PortableAspectContractBasis::from_contract(contract),
            fields,
        })
    }

    fn clear_pending_case_fields(&mut self, id: EntityId, cases: &[&AspectDefinition]) {
        for effect in &mut self.effects {
            match effect {
                Effect::UpdateEntity {
                    entity_id, fields, ..
                } if *entity_id == id => {
                    fields.retain(|field, _| !in_cases(field, cases));
                }
                Effect::PatchOptionalEntityFields {
                    entity_id, fields, ..
                } if *entity_id == id => {
                    fields.retain(|field, _| !in_cases(field, cases));
                }
                _ => {}
            }
        }
    }
}

fn in_cases(
    field: &worth_foundational::facade::AspectFieldLocator,
    cases: &[&AspectDefinition],
) -> bool {
    cases
        .iter()
        .any(|case| case.name() == field.aspect().aspect_key().as_str())
}

fn check_case_slots(
    effects: &[Effect],
    id: EntityId,
    entity: &str,
    cases: &[&AspectDefinition],
    owned: &[usize],
) -> Result<(), Denial> {
    for &slot in owned {
        if !matches!(effects.get(slot), Some(Effect::RemoveAspect { entity: owner, entity_id, contract, .. })
                if *entity_id == id && owner == entity && cases.iter().any(|case| case.name() == contract.key().as_str()))
        {
            return Err(Denial::new(Kind::ConflictingEffectStep, entity));
        }
    }
    for (index, effect) in effects.iter().enumerate() {
        let conflicts = match effect {
            Effect::RemoveAspect {
                entity_id,
                contract,
                ..
            } => {
                *entity_id == id
                    && cases
                        .iter()
                        .any(|case| case.name() == contract.key().as_str())
                    && !owned.contains(&index)
            }
            Effect::DeleteEntity { entity_id } => *entity_id == id,
            _ => false,
        };
        if conflicts {
            return Err(Denial::new(Kind::ConflictingEffectStep, entity));
        }
    }
    Ok(())
}
