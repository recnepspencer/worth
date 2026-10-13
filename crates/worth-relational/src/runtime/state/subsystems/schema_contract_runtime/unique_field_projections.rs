//! Native uniqueness projections admitted once in the installed schema snapshot.
use crate::identity::data::KindId;
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractBinding, LoweredAspectContractPlan,
};
use crate::validation::data::{InvariantRegistration, InvariantRule};
use crate::visibility::materialization::read_records::ProjectionAspectScope;
use std::collections::BTreeMap;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView, FieldKey,
};

#[derive(Debug, Default)]
pub(crate) struct UniqueFieldProjectionCatalog {
    entries: BTreeMap<KindId, BTreeMap<AspectFieldLocator, AdmittedEntityAspectFieldProjection>>,
}

impl UniqueFieldProjectionCatalog {
    pub(super) fn new(
        plans: &AspectContractPlanCatalog,
        registrations: &[InvariantRegistration],
    ) -> Self {
        let mut entries: BTreeMap<
            KindId,
            BTreeMap<AspectFieldLocator, AdmittedEntityAspectFieldProjection>,
        > = BTreeMap::new();
        for registration in registrations {
            let InvariantRule::UniqueEntityAspectField { field_locator } = &registration.rule
            else {
                continue;
            };
            for (kind, plan) in &plans.entity_plans {
                if entries
                    .get(kind)
                    .is_some_and(|fields| fields.contains_key(field_locator))
                {
                    continue;
                }
                if let Some(projection) =
                    AdmittedEntityAspectFieldProjection::for_entity_kind(plan, field_locator)
                {
                    entries
                        .entry(*kind)
                        .or_default()
                        .insert(field_locator.clone(), projection);
                }
            }
        }
        Self { entries }
    }

    pub(crate) fn value<'value>(
        &self,
        kind: KindId,
        locator: &AspectFieldLocator,
        state: impl FnOnce() -> Option<&'value AuthoritativeRecordAspectState>,
    ) -> Option<&'value AspectValue> {
        self.entries
            .get(&kind)?
            .get(locator)?
            .value_from_state(state()?)
    }
}

#[derive(Debug, Clone)]
enum AdmittedEntityAspectFieldProjection {
    ScalarAspect {
        aspect_key: AspectKey,
        projection_scope: ProjectionAspectScope,
    },
    StructField {
        aspect_key: AspectKey,
        field: FieldKey,
        projection_scope: ProjectionAspectScope,
    },
}

impl AdmittedEntityAspectFieldProjection {
    fn for_entity_kind(
        plan: &LoweredAspectContractPlan,
        field_locator: &AspectFieldLocator,
    ) -> Option<Self> {
        let field = single_field_locator_key(field_locator)?;
        let binding = matching_binding(plan, field_locator)?;
        let projection = if binding.targets_entity_scalar_field(field) {
            Self::ScalarAspect {
                aspect_key: binding.aspect_key().clone(),
                projection_scope: ProjectionAspectScope::whole_aspects([binding
                    .aspect_key()
                    .clone()]),
            }
        } else if binding.targets_entity_struct_field(field) {
            Self::StructField {
                aspect_key: binding.aspect_key().clone(),
                field: field.clone(),
                projection_scope: ProjectionAspectScope::fields(
                    binding.aspect_key().clone(),
                    [field.clone()],
                ),
            }
        } else {
            return None;
        };
        projection.admitted_by(binding).then_some(projection)
    }

    fn value_from_state<'a>(
        &self,
        state: &'a AuthoritativeRecordAspectState,
    ) -> Option<&'a AspectValue> {
        match self {
            Self::ScalarAspect { aspect_key, .. } => match state.get(aspect_key)?.view() {
                ContractValidatedAspectValueView::Scalar(value) => Some(value),
                ContractValidatedAspectValueView::Struct(_) => None,
            },
            Self::StructField {
                aspect_key, field, ..
            } => match state.get(aspect_key)?.view() {
                ContractValidatedAspectValueView::Scalar(_) => None,
                ContractValidatedAspectValueView::Struct(value) => value.get(field),
            },
        }
    }

    fn admitted_by(&self, binding: &LoweredAspectContractBinding) -> bool {
        self.projection_scope()
            .requirements()
            .iter()
            .all(|requirement| {
                binding
                    .contract
                    .admits_projection_mask(requirement.mask())
                    .is_ok()
                    && requirement.mask_basis().is_some()
            })
    }

    fn projection_scope(&self) -> &ProjectionAspectScope {
        match self {
            Self::ScalarAspect {
                projection_scope, ..
            }
            | Self::StructField {
                projection_scope, ..
            } => projection_scope,
        }
    }
}

fn matching_binding<'a>(
    plan: &'a LoweredAspectContractPlan,
    field_locator: &AspectFieldLocator,
) -> Option<&'a LoweredAspectContractBinding> {
    plan.executable_bindings
        .iter()
        .find(|binding| binding.aspect_key() == field_locator.aspect().aspect_key())
}

fn single_field_locator_key(field_locator: &AspectFieldLocator) -> Option<&FieldKey> {
    match field_locator.field_path().fields() {
        [field] => Some(field),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
