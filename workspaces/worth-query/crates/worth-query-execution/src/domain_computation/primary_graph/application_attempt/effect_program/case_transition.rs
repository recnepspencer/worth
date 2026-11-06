//! Repeated case writes replace only their own pending sibling-clear slots.
use super::WorthQueryApplicationAttemptDenial as Denial;
use super::WorthQueryApplicationAttemptDenialKind as Kind;
use super::WorthQueryApplicationEffectProgramBuilder as Builder;
use super::WorthQueryApplicationRealizedEffect as Effect;
use crate::domain_computation::primary_graph::fragment_view::ErasedCandidate;
use crate::domain_computation::primary_graph::fragment_view::FragmentDenial;
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::AspectDefinition;
use worth_query_declaration::facade::application_schema::fragment::GroupDefinition;

use worth_relational::facade::identity::EntityId;

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
        self.native_effects()
            .write_case_record(target, selected, group, cases, fields)
    }
}

pub(super) fn in_cases(
    field: &worth_foundational::facade::AspectFieldLocator,
    cases: &[&AspectDefinition],
) -> bool {
    cases
        .iter()
        .any(|case| case.name() == field.aspect().aspect_key().as_str())
}

pub(super) fn check_case_slots(
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
