use crate::contracts::publication::patch::data::patch_detail::PatchDetail;
use crate::contracts::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatch;
use worth_foundational::facade::{
    validate_aspect_value, AspectValue as FoundationalAspectValue, AuthoritativeRecordAspectPatch,
    ContractValidatedAspectArtifact, ContractValidationInput, EntityId as FoundationalEntityId,
    Generation as FoundationalGeneration, InternedString as FoundationalInternedString,
    LocalSlot as FoundationalLocalSlot, PartitionId as FoundationalPartitionId, StructAspectValue,
};
use worth_proof::TransitionOutcome;

use crate::contracts::publication::patch::data::authoritative_patch_envelope::{
    PublishedAuthoritativeRecordPatch, PublishedAuthoritativeRecordPatchOwnerPort as _,
};
use crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange;
use crate::contracts::transactions::data::primitives::RecordRef;

mod changed_bindings;
use changed_bindings::authoritative_patch_filtered_to_changed_bindings;
use super::data::{
    CanonicalAspectDeltaEvidence, CanonicalDeltaError, CanonicalRecordAspectDelta,
    EvaluatedAspectBinding, LifecycleTransitionClass,
};
use super::published_patch_projection::published_patch_from_foundational_patch;
use super::semantic_change_projection::semantic_changes;
use crate::contracts::transactions::data::outcomes::aspect_delta_failure_fields::{
    AspectDeltaPatchConstructionDenial, AspectDeltaPatchValueDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationalPatchFragment {
    pub patch: AuthoritativeRecordAspectPatch,
    pub published: PublishedAuthoritativeRecordPatch,
}

impl std::ops::Deref for FoundationalPatchFragment {
    type Target = PublishedAuthoritativeRecordPatch;

    fn deref(&self) -> &Self::Target {
        &self.published
    }
}

impl CanonicalRecordAspectDelta {
    pub(crate) fn into_foundational_patch_fragment(
        &self,
        detail: PatchDetail,
    ) -> Result<FoundationalPatchFragment, CanonicalDeltaError> {
        let patch = delta_patch(self, |_| true)?;
        FoundationalPatchFragment::from_delta(self, detail, patch)
    }
}

pub(crate) fn authoritative_patch_with_delta_supplements(
    delta: &CanonicalRecordAspectDelta,
    authoritative_patch: super::super::outcomes::mutation_aspect_patch::MutationAspectPatch,
) -> Result<AuthoritativeRecordAspectPatch, CanonicalDeltaError> {
    let changed_authoritative_patch =
        authoritative_patch_filtered_to_changed_bindings(delta, authoritative_patch.as_patch())?;
    let supplemental_patch = delta_patch(delta, |binding| {
        !matches!(
            binding.evidence,
            CanonicalAspectDeltaEvidence::AuthoritativePatch { .. }
        )
    })?;
    if supplemental_patch.is_empty() {
        return Ok(changed_authoritative_patch);
    }

    match AuthoritativeRecordAspectPatch::combine(changed_authoritative_patch, supplemental_patch) {
        TransitionOutcome::Success(patch) => Ok(patch),
        TransitionOutcome::Denied(denial) => {
            Err(CanonicalDeltaError::FoundationalPatchConstruction {
                target: delta.target.clone(),
                denial: AspectDeltaPatchConstructionDenial::FoundationalPatchConstructionDenied(
                    denial,
                ),
            })
        }
    }
}

fn delta_patch(
    delta: &CanonicalRecordAspectDelta,
    include: impl Fn(&EvaluatedAspectBinding) -> bool,
) -> Result<AuthoritativeRecordAspectPatch, CanonicalDeltaError> {
    let mut sets = Vec::new();
    let mut clears = Vec::new();
    for binding in delta
        .evaluated_bindings
        .iter()
        .filter(|binding| binding.changed && include(binding))
    {
        accumulate_patch_action(&delta.target, binding, &mut sets, &mut clears)?;
    }
    if sets.is_empty() && clears.is_empty() {
        return Ok(AuthoritativeRecordAspectPatch::empty());
    }

    match AuthoritativeRecordAspectPatch::whole_aspect(sets, clears) {
        TransitionOutcome::Success(patch) => Ok(patch),
        TransitionOutcome::Denied(denial) => {
            Err(CanonicalDeltaError::FoundationalPatchConstruction {
                target: delta.target.clone(),
                denial: AspectDeltaPatchConstructionDenial::FoundationalPatchConstructionDenied(
                    denial,
                ),
            })
        }
    }
}

impl FoundationalPatchFragment {
    pub(super) fn from_delta(
        delta: &CanonicalRecordAspectDelta,
        detail: PatchDetail,
        patch: AuthoritativeRecordAspectPatch,
    ) -> Result<Self, CanonicalDeltaError> {
        let published = PublishedAuthoritativeRecordPatch {
            target: delta.target.clone(),
            structural_change: delta.structural_change,
            semantic_changes: semantic_changes(delta),
            contains_opaque_aspect: delta.contains_opaque_aspect,
            detail,
            relation_birth: None,
            authoritative_patch: published_patch_for_delta(delta, &patch)?,
        };
        Ok(Self { patch, published })
    }

    pub fn into_published_record(self) -> PublishedAuthoritativeRecordPatch {
        self.published
    }

    pub fn published_record_owned_allocation_capacity_bytes(&self) -> u64 {
        self.published.owned_allocation_capacity_bytes()
    }

    pub fn published_record(&self) -> PublishedAuthoritativeRecordPatch {
        self.published.clone()
    }
}

pub(crate) fn published_patch_for_delta(
    delta: &CanonicalRecordAspectDelta,
    patch: &AuthoritativeRecordAspectPatch,
) -> Result<PublishedAuthoritativePatch, CanonicalDeltaError> {
    published_patch_from_foundational_patch(patch, |aspect_key| {
        delta
            .evaluated_bindings
            .iter()
            .find(|binding| binding.changed && &binding.aspect_key == aspect_key)
            .map(|binding| binding.binding.clone())
    })
    .map_err(
        |aspect_key| CanonicalDeltaError::InvalidLoweredBindingForRecordClass {
            aspect_key,
            detail: "authoritative patch operation has no exact evaluated aspect binding"
                .to_string(),
        },
    )
}

fn accumulate_patch_action(
    target: &RecordRef,
    binding: &EvaluatedAspectBinding,
    sets: &mut Vec<worth_foundational::facade::ContractValidatedAspectArtifact>,
    clears: &mut Vec<worth_foundational::facade::AspectContract>,
) -> Result<(), CanonicalDeltaError> {
    match &binding.evidence {
        CanonicalAspectDeltaEvidence::ScalarAspectValueTransition {
            old_present,
            new_present,
            new_value,
            ..
        } => {
            if *new_present {
                let Some(value) = new_value.clone() else {
                    return Err(CanonicalDeltaError::FoundationalPatchValueValidation {
                        target: target.clone(),
                        aspect_key: binding.aspect_key.clone(),
                        denial: AspectDeltaPatchValueDenial::MissingChangedScalarValue,
                    });
                };
                sets.push(validate_patch_value(target, binding, value)?);
            } else if *old_present {
                clears.push(binding.contract.clone());
            }
        }
        CanonicalAspectDeltaEvidence::StructAspectValueTransition {
            old_present,
            new_present,
            new_value,
            ..
        } => {
            if *new_present {
                let Some(value) = new_value.clone() else {
                    return Err(CanonicalDeltaError::FoundationalPatchValueValidation {
                        target: target.clone(),
                        aspect_key: binding.aspect_key.clone(),
                        denial: AspectDeltaPatchValueDenial::MissingChangedStructValue,
                    });
                };
                sets.push(validate_struct_patch_value(target, binding, value)?);
            } else if *old_present {
                clears.push(binding.contract.clone());
            }
        }
        CanonicalAspectDeltaEvidence::EndpointIdentity { old, new, .. } => {
            if let Some(entity_id) = new {
                let value = FoundationalAspectValue::EntityRef(foundational_entity_id(*entity_id));
                sets.push(validate_patch_value(target, binding, value)?);
            } else if old.is_some() {
                clears.push(binding.contract.clone());
            }
        }
        CanonicalAspectDeltaEvidence::Lifecycle { transition, .. } => {
            let value = FoundationalAspectValue::String(FoundationalInternedString::from(
                lifecycle_label(*transition),
            ));
            sets.push(validate_patch_value(target, binding, value)?);
        }
        CanonicalAspectDeltaEvidence::Structural { change, .. } => {
            let value = FoundationalAspectValue::String(FoundationalInternedString::from(
                structural_change_label(*change),
            ));
            sets.push(validate_patch_value(target, binding, value)?);
        }
        CanonicalAspectDeltaEvidence::AuthoritativePatch { .. } => {
            return Err(CanonicalDeltaError::FoundationalPatchConstruction {
                target: target.clone(),
                denial: AspectDeltaPatchConstructionDenial::AuthoritativePatchEvidenceAlreadyCarriesPatch {
                    aspect_key: binding.aspect_key.clone(),
                },
            });
        }
    }

    Ok(())
}

pub(super) fn validate_patch_value(
    target: &RecordRef,
    binding: &EvaluatedAspectBinding,
    value: FoundationalAspectValue,
) -> Result<ContractValidatedAspectArtifact, CanonicalDeltaError> {
    match validate_aspect_value(&binding.contract, ContractValidationInput::Scalar(value)) {
        TransitionOutcome::Success(artifact) => Ok(artifact),
        TransitionOutcome::Denied(denial) => {
            Err(CanonicalDeltaError::FoundationalPatchValueValidation {
                target: target.clone(),
                aspect_key: binding.aspect_key.clone(),
                denial: AspectDeltaPatchValueDenial::ContractValidationDenied(denial),
            })
        }
    }
}

pub(super) fn validate_struct_patch_value(
    target: &RecordRef,
    binding: &EvaluatedAspectBinding,
    value: StructAspectValue,
) -> Result<ContractValidatedAspectArtifact, CanonicalDeltaError> {
    match validate_aspect_value(&binding.contract, ContractValidationInput::Struct(value)) {
        TransitionOutcome::Success(artifact) => Ok(artifact),
        TransitionOutcome::Denied(denial) => {
            Err(CanonicalDeltaError::FoundationalPatchValueValidation {
                target: target.clone(),
                aspect_key: binding.aspect_key.clone(),
                denial: AspectDeltaPatchValueDenial::ContractValidationDenied(denial),
            })
        }
    }
}

fn foundational_entity_id(
    entity_id: crate::contracts::identity::data::record_ids::EntityId,
) -> FoundationalEntityId {
    FoundationalEntityId {
        partition_id: FoundationalPartitionId(entity_id.partition_id.as_u32()),
        local_slot: FoundationalLocalSlot(entity_id.local_slot_value()),
        generation: FoundationalGeneration(entity_id.generation_value()),
    }
}

fn lifecycle_label(transition: LifecycleTransitionClass) -> &'static str {
    match transition {
        LifecycleTransitionClass::NoTransition => "no_transition",
        LifecycleTransitionClass::Create => "create",
        LifecycleTransitionClass::Delete => "delete",
        LifecycleTransitionClass::RetainForAudit => "retain_for_audit",
    }
}

fn structural_change_label(change: RecordStructuralChange) -> &'static str {
    match change {
        RecordStructuralChange::Created => "create",
        RecordStructuralChange::Updated => "update",
        RecordStructuralChange::Deleted => "delete",
        RecordStructuralChange::RetainedForAudit => "retain_for_audit",
        RecordStructuralChange::MaterializationSuspended => "materialization_suspended",
        RecordStructuralChange::Rematerialized => "rematerialized",
        _ => unreachable!("unhandled Relational contract variant"),
    }
}
