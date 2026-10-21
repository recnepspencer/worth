//! Empty storage is classified by the actual birth plan, never by field counts.
use super::{
    charge, denial::EmptyBindingSummary, OutputPayloadCoverageFailure, UnsupportedPayload,
};
use crate::history::data::CanonicalCommitEnvelope;
use crate::schema::data::LoweredAspectContractPlan;
use worth_execution::MapKernelContext;
use worth_foundational::facade::{AbsenceLaw, AspectBinding, AspectShape};

pub(super) fn validate(
    envelope: &CanonicalCommitEnvelope,
    plan: &LoweredAspectContractPlan,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), OutputPayloadCoverageFailure> {
    let kinds = &envelope.schema_authority.entity_kinds;
    charge(
        context,
        (usize::BITS - kinds.len().leading_zeros()) as usize + 1,
    )?;
    let kind = kinds
        .binary_search_by_key(&plan.kind_id, |kind| kind.kind_id)
        .ok()
        .map(|index| &kinds[index])
        .ok_or("required empty output birth schema kind absent")?;
    if kind.aspect_plan_revision != plan.plan_revision {
        return Err("required empty output birth schema plan differs".into());
    }
    for (ordinal, binding) in plan.executable_bindings.iter().enumerate() {
        charge(context, 1)?;
        if !matches!(binding.target, AspectBinding::EntityField { .. })
            || !matches!(binding.contract.shape(), AspectShape::Struct(_))
            || binding.contract.absence() != AbsenceLaw::Optional
        {
            return Err(OutputPayloadCoverageFailure::Unsupported(
                UnsupportedPayload::EmptyWithBindings {
                    count: plan.executable_bindings.len(),
                    first: EmptyBindingSummary::capture(ordinal, binding, context)?,
                },
            ));
        }
    }
    Ok(())
}
