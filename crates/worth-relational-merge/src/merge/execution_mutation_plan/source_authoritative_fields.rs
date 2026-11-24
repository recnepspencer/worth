use std::collections::BTreeMap;

use worth_foundational::facade::ContractValidatedAspectValueView;

use crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;
use crate::contracts::storage::data::read_records::{EntityReadRecord, RelationReadRecord};
use crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::merge::data::{AdoptSourceRecordPlan, MergeExecutionMutationPlanError};
use worth_foundational::facade::AspectBinding;

pub(super) fn entity_create_fields_from_authoritative_state(
    aspect_plans: &AspectContractPlanCatalog,
    plan: &AdoptSourceRecordPlan,
    entity: &EntityReadRecord,
) -> Result<AspectFieldPatch, MergeExecutionMutationPlanError> {
    let Some(authoritative_state) = entity.authoritative_aspect_state.as_ref() else {
        return Ok(AspectFieldPatch::default());
    };
    let lowered_plan = aspect_plans
        .entity_plans
        .get(&entity.kind.kind_id)
        .ok_or_else(
            || MergeExecutionMutationPlanError::UnsupportedReconcileRecordKind {
                record: plan.source_record.clone(),
                detail:
                    "source entity has authoritative aspect state but no executable aspect plan",
            },
        )?;
    let mut fields = BTreeMap::new();
    for (aspect_key, artifact) in authoritative_state.aspects().entries() {
        let binding = lowered_plan
            .executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == aspect_key)
            .ok_or_else(|| MergeExecutionMutationPlanError::UnsupportedAspectMutationMaterialization {
                record: plan.source_record.clone(),
                aspect_key: aspect_key.clone(),
                detail: "source entity authoritative aspect is not declared by the executable aspect plan",
            })?;
        match artifact.view() {
            ContractValidatedAspectValueView::Scalar(value) => {
                let (
                    AspectBinding::EntityField { field },
                    worth_foundational::AspectShape::Scalar(_),
                ) = (&binding.target, binding.contract.shape())
                else {
                    return Err(MergeExecutionMutationPlanError::UnsupportedAspectMutationMaterialization {
                        record: plan.source_record.clone(),
                        aspect_key: aspect_key.clone(),
                        detail: "source entity scalar authoritative aspect is not backed by an entity scalar field binding",
                    });
                };
                fields.insert(
                    crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                        aspect_key.clone(),
                        field.clone(),
                    ),
                    value.clone(),
                );
            }
            ContractValidatedAspectValueView::Struct(struct_value) => {
                if !matches!(
                    (&binding.target, binding.contract.shape()),
                    (
                        AspectBinding::EntityField { .. },
                        worth_foundational::AspectShape::Struct(_)
                    )
                ) {
                    return Err(MergeExecutionMutationPlanError::UnsupportedAspectMutationMaterialization {
                        record: plan.source_record.clone(),
                        aspect_key: aspect_key.clone(),
                        detail: "source entity struct authoritative aspect is not backed by an entity struct field binding",
                    });
                }
                for (field, value) in struct_value.fields() {
                    fields.insert(
                        crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                            aspect_key.clone(),
                            field.clone(),
                        ),
                        value.clone(),
                    );
                }
            }
        }
    }
    Ok(AspectFieldPatch::from(fields))
}

pub(super) fn relation_create_fields_from_authoritative_state(
    aspect_plans: &AspectContractPlanCatalog,
    plan: &AdoptSourceRecordPlan,
    relation: &RelationReadRecord,
) -> Result<AspectFieldPatch, MergeExecutionMutationPlanError> {
    let Some(authoritative_state) = relation.authoritative_aspect_state.as_ref() else {
        return Ok(AspectFieldPatch::default());
    };
    let lowered_plan = aspect_plans
        .relation_plans
        .get(&relation.kind.kind_id)
        .ok_or_else(
            || MergeExecutionMutationPlanError::UnsupportedReconcileRecordKind {
                record: plan.source_record.clone(),
                detail:
                    "source relation has authoritative aspect state but no executable aspect plan",
            },
        )?;
    let mut fields = BTreeMap::new();
    for (aspect_key, artifact) in authoritative_state.aspects().entries() {
        let Some(binding) = lowered_plan
            .executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == aspect_key)
        else {
            continue;
        };
        match (&binding.target, binding.contract.shape(), artifact.view()) {
            (
                AspectBinding::RelationField { field },
                worth_foundational::AspectShape::Scalar(_),
                ContractValidatedAspectValueView::Scalar(value),
            ) => {
                fields.insert(
                    crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                        aspect_key.clone(),
                        field.clone(),
                    ),
                    value.clone(),
                );
            }
            (
                AspectBinding::RelationField { .. },
                worth_foundational::AspectShape::Struct(_),
                ContractValidatedAspectValueView::Struct(struct_value),
            ) => {
                for (field, value) in struct_value.fields() {
                    fields.insert(
                        crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
                            aspect_key.clone(),
                            field.clone(),
                        ),
                        value.clone(),
                    );
                }
            }
            (
                AspectBinding::RelationSourceEndpoint | AspectBinding::RelationTargetEndpoint,
                _,
                _,
            ) => {}
            _ => {
                return Err(MergeExecutionMutationPlanError::UnsupportedAspectMutationMaterialization {
                    record: plan.source_record.clone(),
                    aspect_key: aspect_key.clone(),
                    detail: "source relation authoritative aspect shape does not match relation field binding",
                });
            }
        }
    }
    Ok(AspectFieldPatch::from(fields))
}
