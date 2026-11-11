use smallvec::SmallVec;

use crate::publication::patch::data::{ordered_aspect_keys, RecordStructuralChange};
use crate::schema::data::{AspectBinding, LoweredAspectContractPlan};
use crate::transactions::data::RecordRef;

use super::data::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
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
        retained_relation_endpoints: None,
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
    borrowed_authoritative_patch_binding_evidence(binding, structural_change, patch)
        .map(|evidence| evidence.into_owned(binding.aspect_key()))
}

pub(super) fn borrowed_authoritative_patch_binding_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
    patch: &'a worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
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

fn structural_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
    matches!(
        &binding.target,
        AspectBinding::StructuralRegion
            | AspectBinding::StructuralPartition
            | AspectBinding::StructuralFacet
    )
    .then(|| super::BorrowedAspectDeltaEvidence::Structural {
        change: structural_change,
    })
}

fn lifecycle_structural_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    structural_change: RecordStructuralChange,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
    if !matches!(&binding.target, AspectBinding::LifecycleTransition) {
        return None;
    }
    let transition = lifecycle_transition(structural_change);
    (transition != super::data::LifecycleTransitionClass::NoTransition)
        .then_some(super::BorrowedAspectDeltaEvidence::Lifecycle { transition })
}

fn whole_aspect_set_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    patch: &'a worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
    let (_, value) = patch
        .whole_aspect_sets()
        .find(|(key, _)| *key == binding.contract.key())?;
    Some(super::BorrowedAspectDeltaEvidence::AuthoritativePatch {
        operation: super::BorrowedPatchOperation::WholeAspectSet { value },
    })
}

fn whole_aspect_clear_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    patch: &'a worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
    patch
        .whole_aspect_clears()
        .any(|key| key == binding.contract.key())
        .then_some(super::BorrowedAspectDeltaEvidence::AuthoritativePatch {
            operation: super::BorrowedPatchOperation::WholeAspectClear {
                contract: &binding.contract,
            },
        })
}

fn field_level_patch_evidence<'a>(
    binding: &'a crate::schema::data::LoweredAspectContractBinding,
    patch: &'a worth_foundational::facade::AuthoritativeRecordAspectPatch,
) -> Option<super::BorrowedAspectDeltaEvidence<'a>> {
    let (_, field_patch) = patch
        .field_patches()
        .find(|(key, _)| *key == binding.contract.key())?;
    Some(super::BorrowedAspectDeltaEvidence::AuthoritativePatch {
        operation: super::BorrowedPatchOperation::FieldLevelPatch { patch: field_patch },
    })
}
