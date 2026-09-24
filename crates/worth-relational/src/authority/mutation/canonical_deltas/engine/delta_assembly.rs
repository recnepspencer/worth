use smallvec::SmallVec;

use crate::publication::patch::data::{ordered_aspect_keys, RecordStructuralChange};
use crate::schema::data::LoweredAspectContractPlan;
use crate::transactions::data::RecordRef;

use crate::authority::mutation::canonical_deltas::data::{
    BindingEvaluationContext, CanonicalDeltaError, CanonicalRecordAspectDelta,
    EvaluatedAspectBinding,
};
use crate::authority::mutation::canonical_deltas::materialized_state::evaluate_authoritative_binding_delta;

pub(super) fn assemble_delta(
    target: RecordRef,
    kind_id: crate::identity::data::KindId,
    plan: &LoweredAspectContractPlan,
    structural_change: RecordStructuralChange,
    evaluated_bindings: SmallVec<[EvaluatedAspectBinding; 4]>,
) -> CanonicalRecordAspectDelta {
    let changed_aspects = ordered_aspect_keys(
        evaluated_bindings
            .iter()
            .filter(|binding| binding.changed)
            .map(|binding| binding.aspect_key.clone()),
    );
    let contains_opaque_aspect = evaluated_bindings.iter().any(|binding| {
        binding.changed
            && matches!(
                binding.aspect_shape,
                worth_foundational::AspectShape::Opaque(_)
            )
    });
    CanonicalRecordAspectDelta {
        target,
        kind_id,
        plan_revision: plan.plan_revision,
        structural_change,
        changed_aspects,
        evaluated_bindings,
        contains_opaque_aspect,
    }
}

pub(super) fn evaluate_bindings(
    plan: &LoweredAspectContractPlan,
    context: BindingEvaluationContext<'_>,
) -> Result<SmallVec<[EvaluatedAspectBinding; 4]>, CanonicalDeltaError> {
    let mut evaluated = SmallVec::new();
    for binding in &plan.executable_bindings {
        let (evidence, changed) = evaluate_authoritative_binding_delta(binding, context)?;
        evaluated.push(EvaluatedAspectBinding {
            aspect_key: binding.aspect_key().clone(),
            contract: binding.contract.clone(),
            binding: binding.target.clone(),
            changed,
            aspect_shape: binding.aspect_shape(),
            evidence,
            field_revision_changes: None,
        });
    }
    Ok(evaluated)
}
