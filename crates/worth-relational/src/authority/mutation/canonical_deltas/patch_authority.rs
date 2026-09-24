use smallvec::SmallVec;

use crate::publication::patch::data::{ordered_aspect_keys, RecordStructuralChange};
use crate::schema::data::{AspectBinding, LoweredAspectContractPlan};
use crate::transactions::data::RecordRef;
use worth_foundational::facade::{AspectLocator, AspectValueLocator, LocatorAuthority};

use super::data::{
    AuthoritativePatchDeltaOperation, CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta,
    EvaluatedAspectBinding,
};
use super::lifecycle_transition_evidence::lifecycle_transition;

pub(super) fn evaluate_authoritative_patch_delta(
    target: RecordRef,
    kind_id: crate::identity::data::KindId,
    plan: &LoweredAspectContractPlan,
    structural_change: RecordStructuralChange,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> CanonicalRecordAspectDelta {
    let evaluated_bindings = authoritative_patch_evaluated_bindings(plan, structural_change, patch);
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

fn authoritative_patch_evaluated_bindings(
    plan: &LoweredAspectContractPlan,
    structural_change: RecordStructuralChange,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> SmallVec<[EvaluatedAspectBinding; 4]> {
    let mut evaluated = SmallVec::new();
    for binding in &plan.executable_bindings {
        if let Some(evidence) =
            authoritative_patch_binding_evidence(binding, structural_change, patch)
        {
            evaluated.push(EvaluatedAspectBinding {
                aspect_key: binding.aspect_key().clone(),
                contract: binding.contract.clone(),
                binding: binding.target.clone(),
                changed: true,
                aspect_shape: binding.aspect_shape(),
                evidence,
                field_revision_changes: None,
            });
        }
    }
    evaluated
}

pub(super) fn authoritative_patch_binding_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<CanonicalAspectDeltaEvidence> {
    match &binding.target {
        AspectBinding::StructuralRegion
        | AspectBinding::StructuralPartition
        | AspectBinding::StructuralFacet => structural_evidence(binding, structural_change),
        AspectBinding::LifecycleTransition => {
            lifecycle_structural_evidence(binding, structural_change)
        }
        _ => whole_aspect_set_evidence(binding, patch)
            .or_else(|| whole_aspect_clear_evidence(binding, patch))
            .or_else(|| field_level_patch_evidence(binding, patch)),
    }
}

fn structural_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
) -> Option<CanonicalAspectDeltaEvidence> {
    matches!(
        &binding.target,
        AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet
    )
    .then(|| CanonicalAspectDeltaEvidence::Structural {
        locator: authoritative_value_locator(binding),
        change: structural_change,
    })
}

fn lifecycle_structural_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
) -> Option<CanonicalAspectDeltaEvidence> {
    if !matches!(&binding.target, AspectBinding::LifecycleTransition) {
        return None;
    }
    let transition = lifecycle_transition(structural_change);
    (transition != super::data::LifecycleTransitionClass::NoTransition).then(|| {
        CanonicalAspectDeltaEvidence::Lifecycle {
            locator: authoritative_value_locator(binding),
            transition,
        }
    })
}

fn whole_aspect_set_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<CanonicalAspectDeltaEvidence> {
    let (_, value) = patch
        .whole_aspect_sets()
        .find(|(key, _)| *key == binding.contract.key())?;
    Some(CanonicalAspectDeltaEvidence::AuthoritativePatch {
        locator: authoritative_value_locator(binding),
        operation: AuthoritativePatchDeltaOperation::WholeAspectSet {
            value: value.clone(),
        },
    })
}

fn whole_aspect_clear_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<CanonicalAspectDeltaEvidence> {
    patch
        .whole_aspect_clears()
        .any(|key| key == binding.contract.key())
        .then(|| CanonicalAspectDeltaEvidence::AuthoritativePatch {
            locator: authoritative_value_locator(binding),
            operation: AuthoritativePatchDeltaOperation::WholeAspectClear {
                contract: binding.contract.clone(),
            },
        })
}

fn field_level_patch_evidence(
    binding: &crate::schema::data::LoweredAspectContractBinding,
    patch: &worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<CanonicalAspectDeltaEvidence> {
    let (_, field_patch) = patch
        .field_patches()
        .find(|(key, _)| *key == binding.contract.key())?;
    Some(CanonicalAspectDeltaEvidence::AuthoritativePatch {
        locator: authoritative_value_locator(binding),
        operation: AuthoritativePatchDeltaOperation::FieldLevelPatch {
            patch: field_patch.clone(),
        },
    })
}

fn authoritative_value_locator(
    binding: &crate::schema::data::LoweredAspectContractBinding,
) -> AspectValueLocator {
    AspectValueLocator::whole_aspect(AspectLocator::new(
        LocatorAuthority::Authoritative,
        binding.contract.key().clone(),
    ))
}
