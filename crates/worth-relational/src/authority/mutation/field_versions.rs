use worth_foundational::facade::{
    AspectBinding, AspectShape, ContractValidatedAspectValueView, FieldKey, StructAspectValue,
};

use crate::authority::mutation::canonical_deltas::{
    AuthoritativePatchDeltaOperation, CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta,
    EvaluatedAspectBinding,
};
use crate::identity::data::VersionId;
use crate::storage::data::{RelationalFieldPresence, RelationalFieldRevision};
use crate::storage::overlay::WorkingState;
use crate::storage::substrate::{partition_of, slot_of, EntityRecordKind, RelationRecordKind};
use crate::symbols::data::StringInterner;
use crate::transactions::data::RecordRef;

/// Stamp owner-native field transitions on the same staged root that receives
/// the canonical aspect delta. Unknown restored columns stay unknown until a
/// checkpoint image version can carry their prior revisions.
pub(super) fn write_field_versions_for_delta(
    staged: &mut WorkingState,
    delta: &CanonicalRecordAspectDelta,
    version: VersionId,
    symbols: &mut StringInterner,
) {
    for binding in &delta.evaluated_bindings {
        let fields = changed_fields(binding, delta.structural_change);
        if fields.is_empty() {
            continue;
        }
        let aspect = symbols.intern(binding.aspect_key.as_str());
        match delta.target {
            RecordRef::Entity(entity) => {
                let partition = staged.get_partition_mut(partition_of::<EntityRecordKind>(&entity));
                let Some(revisions) =
                    partition
                        .entity_arena
                        .field_revisions_at_mut(slot_of::<EntityRecordKind>(&entity))
                else {
                    continue;
                };
                for (field, presence) in fields {
                    revisions.insert(
                        (aspect, symbols.intern(field.as_str())),
                        RelationalFieldRevision::new(version, presence),
                    );
                }
            }
            RecordRef::Relation(relation) => {
                let partition =
                    staged.get_partition_mut(partition_of::<RelationRecordKind>(&relation));
                let Some(revisions) =
                    partition
                        .relation_arena
                        .field_revisions_at_mut(slot_of::<RelationRecordKind>(&relation))
                else {
                    continue;
                };
                for (field, presence) in fields {
                    revisions.insert(
                        (aspect, symbols.intern(field.as_str())),
                        RelationalFieldRevision::new(version, presence),
                    );
                }
            }
        }
    }
}

pub(super) fn changed_fields(
    binding: &EvaluatedAspectBinding,
    structural: crate::publication::patch::data::RecordStructuralChange,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    if let Some(changes) = &binding.field_revision_changes {
        return changes.clone();
    }
    field_changes_for_evidence(
        &binding.binding,
        &binding.aspect_shape,
        &binding.evidence,
        structural,
    )
}

pub(super) fn field_changes_for_evidence(
    binding: &AspectBinding,
    aspect_shape: &AspectShape,
    evidence: &CanonicalAspectDeltaEvidence,
    structural: crate::publication::patch::data::RecordStructuralChange,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    use RelationalFieldPresence::{Absent, Present};
    match (aspect_shape, evidence) {
        (
            AspectShape::Struct(shape),
            CanonicalAspectDeltaEvidence::StructAspectValueTransition {
                old_value,
                new_value,
                ..
            },
        ) => shape
            .fields()
            .iter()
            .filter_map(|field| {
                let old = old_value.as_ref().and_then(|value| value.get(field.key()));
                let new = new_value.as_ref().and_then(|value| value.get(field.key()));
                (old != new || starts_new_record(structural)).then(|| {
                    (
                        field.key().clone(),
                        if new.is_some() { Present } else { Absent },
                    )
                })
            })
            .collect(),
        (
            AspectShape::Struct(shape),
            CanonicalAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => patch_fields(shape, operation),
        (
            AspectShape::Scalar(_),
            CanonicalAspectDeltaEvidence::ScalarAspectValueTransition {
                old_value,
                new_value,
                ..
            },
        ) => bound_field(binding)
            .filter(|_| old_value != new_value || starts_new_record(structural))
            .map(|field| {
                vec![(
                    field.clone(),
                    if new_value.is_some() { Present } else { Absent },
                )]
            })
            .unwrap_or_default(),
        (
            AspectShape::Scalar(_),
            CanonicalAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => bound_field(binding)
            .map(|field| {
                vec![(
                    field.clone(),
                    match operation {
                        AuthoritativePatchDeltaOperation::WholeAspectClear { .. } => Absent,
                        _ => Present,
                    },
                )]
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn patch_fields(
    shape: &worth_foundational::facade::StructAspectShape,
    operation: &AuthoritativePatchDeltaOperation,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    use RelationalFieldPresence::{Absent, Present};
    match operation {
        AuthoritativePatchDeltaOperation::FieldLevelPatch { patch } => patch
            .field_sets()
            .map(|(field, _)| (field.clone(), Present))
            .chain(patch.field_clears().map(|field| (field.clone(), Absent)))
            .collect(),
        AuthoritativePatchDeltaOperation::WholeAspectClear { .. } => shape
            .fields()
            .iter()
            .map(|field| (field.key().clone(), Absent))
            .collect(),
        AuthoritativePatchDeltaOperation::WholeAspectSet { value } => {
            let ContractValidatedAspectValueView::Struct(value) = value.view() else {
                return Vec::new();
            };
            all_struct_fields(shape, value)
        }
    }
}

fn all_struct_fields(
    shape: &worth_foundational::facade::StructAspectShape,
    value: &StructAspectValue,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    shape
        .fields()
        .iter()
        .map(|field| {
            (
                field.key().clone(),
                if value.get(field.key()).is_some() {
                    RelationalFieldPresence::Present
                } else {
                    RelationalFieldPresence::Absent
                },
            )
        })
        .collect()
}

fn bound_field(binding: &AspectBinding) -> Option<&FieldKey> {
    match binding {
        AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
            Some(field)
        }
        _ => None,
    }
}

fn starts_new_record(change: crate::publication::patch::data::RecordStructuralChange) -> bool {
    matches!(
        change,
        crate::publication::patch::data::RecordStructuralChange::Created
            | crate::publication::patch::data::RecordStructuralChange::Rematerialized
    )
}
