use crate::publication::patch::data::{PublishedAuthoritativeAspectChange, RecordStructuralChange};

use super::data::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
    LifecycleTransitionClass,
};

pub(crate) fn semantic_changes(
    delta: &CanonicalRecordAspectDelta,
) -> Vec<PublishedAuthoritativeAspectChange> {
    let mut changes = Vec::new();
    for binding in delta
        .evaluated_bindings
        .iter()
        .filter(|binding| binding.changed)
    {
        append_semantic_changes(binding, &mut changes);
    }
    changes.sort_by_key(PublishedAuthoritativeAspectChange::canonical_key);
    changes.dedup();
    changes
}

fn append_semantic_changes(
    binding: &EvaluatedAspectBinding,
    changes: &mut Vec<PublishedAuthoritativeAspectChange>,
) {
    use worth_foundational::facade::{AuthoritativeAspectChangeKind as Kind, CanonicalFieldPath};

    let push = |kind, field_path, changes: &mut Vec<_>| {
        changes.push(PublishedAuthoritativeAspectChange::exact(
            binding.aspect_key.clone(),
            binding.contract.identity(),
            binding.contract.revision(),
            binding.binding.clone(),
            kind,
            field_path,
        ));
    };
    if matches!(
        binding.aspect_shape,
        worth_foundational::AspectShape::Opaque(_)
    ) {
        push(Kind::Opaque, None, changes);
        return;
    }

    match &binding.evidence {
        CanonicalAspectDeltaEvidence::ScalarAspectValueTransition { new_present, .. }
        | CanonicalAspectDeltaEvidence::StructAspectValueTransition { new_present, .. } => push(
            if *new_present {
                Kind::WholeAspectSet
            } else {
                Kind::WholeAspectClear
            },
            None,
            changes,
        ),
        CanonicalAspectDeltaEvidence::EndpointIdentity { .. } => push(
            match binding.binding {
                worth_foundational::facade::AspectBinding::RelationSourceEndpoint => {
                    Kind::RelationSourceEndpoint
                }
                worth_foundational::facade::AspectBinding::RelationTargetEndpoint => {
                    Kind::RelationTargetEndpoint
                }
                _ => Kind::WholeAspectSet,
            },
            None,
            changes,
        ),
        CanonicalAspectDeltaEvidence::Lifecycle { transition, .. } => {
            let kind = match transition {
                LifecycleTransitionClass::Create => Kind::LifecycleCreate,
                LifecycleTransitionClass::Delete => Kind::LifecycleDelete,
                LifecycleTransitionClass::RetainForAudit => Kind::LifecycleRetainForAudit,
                LifecycleTransitionClass::NoTransition => return,
            };
            push(kind, None, changes);
        }
        CanonicalAspectDeltaEvidence::Structural { change, .. } => {
            let kind = match change {
                RecordStructuralChange::Created => Kind::StructuralCreate,
                RecordStructuralChange::Updated => Kind::StructuralUpdate,
                RecordStructuralChange::Deleted => Kind::StructuralDelete,
                RecordStructuralChange::RetainedForAudit => Kind::StructuralRetainForAudit,
                RecordStructuralChange::MaterializationSuspended => {
                    Kind::StructuralMaterializationSuspended
                }
                RecordStructuralChange::Rematerialized => Kind::StructuralRematerialized,
            };
            push(kind, None, changes);
        }
        CanonicalAspectDeltaEvidence::AuthoritativePatch { operation, .. } => match operation {
            super::data::AuthoritativePatchDeltaOperation::WholeAspectSet { .. } => {
                push(whole_operation_kind(&binding.binding, true), None, changes)
            }
            super::data::AuthoritativePatchDeltaOperation::WholeAspectClear { .. } => {
                push(whole_operation_kind(&binding.binding, false), None, changes)
            }
            super::data::AuthoritativePatchDeltaOperation::FieldLevelPatch { patch } => {
                for (field, _) in patch.field_sets() {
                    push(
                        Kind::FieldSet,
                        Some(CanonicalFieldPath::single(field.clone())),
                        changes,
                    );
                }
                for field in patch.field_clears() {
                    push(
                        Kind::FieldClear,
                        Some(CanonicalFieldPath::single(field.clone())),
                        changes,
                    );
                }
            }
        },
    }
}

fn whole_operation_kind(
    binding: &worth_foundational::facade::AspectBinding,
    is_set: bool,
) -> worth_foundational::facade::AuthoritativeAspectChangeKind {
    use worth_foundational::facade::{AspectBinding, AuthoritativeAspectChangeKind as Kind};
    match binding {
        AspectBinding::RelationSourceEndpoint => Kind::RelationSourceEndpoint,
        AspectBinding::RelationTargetEndpoint => Kind::RelationTargetEndpoint,
        _ if is_set => Kind::WholeAspectSet,
        _ => Kind::WholeAspectClear,
    }
}
