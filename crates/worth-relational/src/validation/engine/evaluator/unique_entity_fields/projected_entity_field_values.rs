use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, AuthoritativeRecordAspectState,
    ContractValidatedAspectValueView, FieldKey,
};

use crate::identity::data::KindId;
use crate::schema::data::{LoweredAspectContractBinding, LoweredAspectContractPlan};
use crate::validation::engine::state_view::VisibleEntityMetadata;
use crate::visibility::materialization::read_records::ProjectionAspectScope;

use super::super::super::context::InvariantExecutionContext;

#[derive(Debug, Clone)]
pub(super) struct ProjectedEntityAspectFieldValue {
    pub(super) value: AspectValue,
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

pub(super) fn projected_entity_aspect_field_value_for_metadata(
    context: &InvariantExecutionContext<'_, '_>,
    metadata: &VisibleEntityMetadata,
    field_locator: &AspectFieldLocator,
) -> Option<ProjectedEntityAspectFieldValue> {
    let projection = AdmittedEntityAspectFieldProjection::for_entity_kind(
        context,
        metadata.kind_id,
        field_locator,
    )?;
    let state = context
        .enforcement_state_view()
        .entity_aspect_state(metadata.entity_id)?;
    projection
        .value_from_state(state)
        .cloned()
        .map(|value| ProjectedEntityAspectFieldValue { value })
}

impl AdmittedEntityAspectFieldProjection {
    fn for_entity_kind(
        context: &InvariantExecutionContext<'_, '_>,
        kind_id: KindId,
        field_locator: &AspectFieldLocator,
    ) -> Option<Self> {
        let field = single_field_locator_key(field_locator)?;
        let plan = context.entity_aspect_plan(kind_id)?;
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
