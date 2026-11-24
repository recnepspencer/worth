use std::sync::Arc;

use crate::merge::aspect_components::{binding_component_from_visible_record, VisibleRecordSide};
use crate::merge::aspect_witness_digest::canonical_aspect_witness_digest;
use crate::merge::data::{
    ExecutableAspectPlan, LoweredAspectExecutionIntent, MaterializedAspectValue,
    MaterializedAspectValueEvidence, MergeExecutionCompilationError, MergeValueMaterialization,
    MergeValueSourceSide,
};
use crate::schema::data::AspectContractPlanCatalog;
use crate::schema::data::LoweredAspectContractBinding;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn compile_executable_aspect_plans(
    aspect_plans: &AspectContractPlanCatalog,
    source_record: &crate::merge::data::VisibleMergeRecord,
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
) -> Result<Arc<[ExecutableAspectPlan]>, MergeExecutionCompilationError> {
    let mut plans = Vec::with_capacity(lowered_record.aspect_outcomes.len());
    for aspect in lowered_record.aspect_outcomes.iter() {
        let execution_intent = aspect.execution_intent.ok_or_else(|| {
            MergeExecutionCompilationError::MissingAspectExecutionIntent {
                record: lowered_record.record.clone(),
                aspect_key: aspect.aspect_key.clone(),
            }
        })?;
        let authorized_values = aspect.authorized_values.ok_or_else(|| {
            MergeExecutionCompilationError::MissingAuthorizedAspectValues {
                record: lowered_record.record.clone(),
                aspect_key: aspect.aspect_key.clone(),
            }
        })?;
        let plan = match execution_intent {
            LoweredAspectExecutionIntent::AdoptSourceValue { .. } => {
                ExecutableAspectPlan::AdoptSourceValue {
                    aspect_key: aspect.aspect_key.clone(),
                    source_value: crate::merge::data::aspect_reference(
                        MergeValueSourceSide::Source,
                        source_record.record_ref.clone(),
                        aspect.aspect_key.clone(),
                    ),
                }
            }
            LoweredAspectExecutionIntent::PreserveSharedValue { .. } => {
                let witness_digest = aspect_shared_witness_digest(
                    aspect_plans,
                    source_record,
                    &aspect.aspect_key,
                    lowered_record.record.clone(),
                )?;
                ExecutableAspectPlan::PreserveSharedValue {
                    aspect_key: aspect.aspect_key.clone(),
                    shared_value: MaterializedAspectValue {
                        policy: MergeValueMaterialization::EqualityWitnessDigest,
                        evidence: MaterializedAspectValueEvidence::EqualityWitnessDigest(
                            witness_digest,
                        ),
                    },
                }
            }
            LoweredAspectExecutionIntent::ReconcileVisibleValues { .. } => {
                let source_value = aspect_materialized_reference(
                    authorized_values.source,
                    MergeValueSourceSide::Source,
                    lowered_record.record.clone(),
                    aspect.aspect_key.clone(),
                );
                let target_value = aspect_materialized_reference(
                    authorized_values.target,
                    MergeValueSourceSide::Target,
                    lowered_record
                        .target_record
                        .clone()
                        .unwrap_or_else(|| lowered_record.record.clone()),
                    aspect.aspect_key.clone(),
                );
                let base_value = aspect_materialized_reference(
                    authorized_values.base,
                    MergeValueSourceSide::Base,
                    lowered_record.record.clone(),
                    aspect.aspect_key.clone(),
                );
                let resolved_value = resolved_materialized_value(lowered_record, aspect);
                ExecutableAspectPlan::ReconcileValue {
                    aspect_key: aspect.aspect_key.clone(),
                    resolved_value,
                    source_value,
                    target_value,
                    base_value,
                }
            }
        };
        plans.push(plan);
    }
    Ok(Arc::from(plans))
}

fn resolved_materialized_value(
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
    aspect: &crate::merge::data::LoweredAspectOutcome,
) -> Option<MaterializedAspectValue> {
    match aspect.resolved_value_strategy.as_ref()? {
        crate::merge::data::MergeResolvedAspectValueStrategy::SourceVisibleValue => {
            Some(crate::merge::data::aspect_reference(
                MergeValueSourceSide::Source,
                lowered_record.record.clone(),
                aspect.aspect_key.clone(),
            ))
        }
        crate::merge::data::MergeResolvedAspectValueStrategy::TargetVisibleValue => {
            Some(crate::merge::data::aspect_reference(
                MergeValueSourceSide::Target,
                lowered_record
                    .target_record
                    .clone()
                    .unwrap_or_else(|| lowered_record.record.clone()),
                aspect.aspect_key.clone(),
            ))
        }
        crate::merge::data::MergeResolvedAspectValueStrategy::BaseVisibleValue => {
            Some(crate::merge::data::aspect_reference(
                MergeValueSourceSide::Base,
                lowered_record.record.clone(),
                aspect.aspect_key.clone(),
            ))
        }
        crate::merge::data::MergeResolvedAspectValueStrategy::InlineAspectValue(value) => {
            Some(MaterializedAspectValue {
                policy: MergeValueMaterialization::EagerInlineAspectValue,
                evidence: MaterializedAspectValueEvidence::InlineAspectValue(value.clone()),
            })
        }
    }
}

fn aspect_materialized_reference(
    usage: crate::merge::data::AuthorizedAspectValueUsage,
    side: MergeValueSourceSide,
    record: RecordRef,
    aspect_key: worth_foundational::facade::AspectKey,
) -> Option<MaterializedAspectValue> {
    match usage {
        crate::merge::data::AuthorizedAspectValueUsage::NotAuthorized => None,
        crate::merge::data::AuthorizedAspectValueUsage::ConsumeVisibleValue
        | crate::merge::data::AuthorizedAspectValueUsage::ConsumeBaseValue => Some(
            crate::merge::data::aspect_reference(side, record, aspect_key),
        ),
        crate::merge::data::AuthorizedAspectValueUsage::EqualityWitnessOnly => None,
    }
}

fn aspect_shared_witness_digest(
    aspect_plans: &AspectContractPlanCatalog,
    source_record: &crate::merge::data::VisibleMergeRecord,
    aspect_key: &worth_foundational::facade::AspectKey,
    record: RecordRef,
) -> Result<String, MergeExecutionCompilationError> {
    let binding =
        aspect_binding_for_record(aspect_plans, source_record, aspect_key).ok_or_else(|| {
            MergeExecutionCompilationError::MissingAspectBinding {
                record: record.clone(),
                aspect_key: aspect_key.clone(),
            }
        })?;
    let source_component =
        binding_component_from_visible_record(source_record, binding, VisibleRecordSide::Source)
            .ok_or_else(
                || MergeExecutionCompilationError::MissingAspectValueWitness {
                    record: record.clone(),
                    aspect_key: aspect_key.clone(),
                },
            )?;
    let target_component =
        binding_component_from_visible_record(source_record, binding, VisibleRecordSide::Target)
            .ok_or_else(
                || MergeExecutionCompilationError::MissingAspectValueWitness {
                    record: record.clone(),
                    aspect_key: aspect_key.clone(),
                },
            )?;
    Ok(canonical_aspect_witness_digest(
        aspect_key,
        &source_component,
        &target_component,
    ))
}

fn aspect_binding_for_record<'a>(
    aspect_plans: &'a AspectContractPlanCatalog,
    source_record: &crate::merge::data::VisibleMergeRecord,
    aspect_key: &worth_foundational::facade::AspectKey,
) -> Option<&'a LoweredAspectContractBinding> {
    let plan =
        crate::merge::aspect_plan_lookup::lowered_plan_for_record(aspect_plans, source_record)?;
    plan.executable_bindings
        .iter()
        .find(|binding| binding.aspect_key() == aspect_key)
}
