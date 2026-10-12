use worth_foundational::facade::{
    AspectBinding, AspectShape, ContractValidatedAspectValueView, FieldKey,
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

pub(crate) fn changed_fields(
    binding: &EvaluatedAspectBinding,
    structural: crate::publication::patch::data::RecordStructuralChange,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    let mut fields = Vec::new();
    visit_changed_fields(binding, structural, |field, presence| {
        fields.push((field.clone(), presence));
        Ok::<(), std::convert::Infallible>(())
    })
    .unwrap();
    fields
}

/// Visits the exact owner-certified changes without cloning their keys or array.
pub(crate) fn visit_changed_fields<E>(
    binding: &EvaluatedAspectBinding,
    structural: crate::publication::patch::data::RecordStructuralChange,
    visit: impl FnMut(&FieldKey, RelationalFieldPresence) -> Result<(), E>,
) -> Result<(), E> {
    if let Some(changes) = &binding.field_revision_changes {
        let mut visit = visit;
        for (field, presence) in changes {
            visit(field, *presence)?;
        }
        return Ok(());
    }
    visit_field_changes_for_evidence(
        &binding.binding,
        &binding.aspect_shape,
        &binding.evidence,
        structural,
        visit,
    )
}

pub(super) fn field_changes_for_evidence(
    binding: &AspectBinding,
    shape: &AspectShape,
    evidence: &CanonicalAspectDeltaEvidence,
    structural: crate::publication::patch::data::RecordStructuralChange,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    let mut fields = Vec::new();
    visit_field_changes_for_evidence(binding, shape, evidence, structural, |field, presence| {
        fields.push((field.clone(), presence));
        Ok::<(), std::convert::Infallible>(())
    })
    .unwrap();
    fields
}

fn visit_field_changes_for_evidence<E>(
    binding: &AspectBinding,
    shape: &AspectShape,
    evidence: &CanonicalAspectDeltaEvidence,
    structural: crate::publication::patch::data::RecordStructuralChange,
    mut visit: impl FnMut(&FieldKey, RelationalFieldPresence) -> Result<(), E>,
) -> Result<(), E> {
    use RelationalFieldPresence::{Absent, Present};
    match (shape, evidence) {
        (
            AspectShape::Struct(shape),
            CanonicalAspectDeltaEvidence::StructAspectValueTransition {
                old_value,
                new_value,
                ..
            },
        ) => {
            for field in shape.fields() {
                let old = old_value.as_ref().and_then(|value| value.get(field.key()));
                let new = new_value.as_ref().and_then(|value| value.get(field.key()));
                if old != new || starts_new_record(structural) {
                    visit(field.key(), if new.is_some() { Present } else { Absent })?;
                }
            }
        }
        (
            AspectShape::Scalar(_),
            CanonicalAspectDeltaEvidence::ScalarAspectValueTransition {
                old_value,
                new_value,
                ..
            },
        ) => {
            if let Some(field) = bound_field(binding)
                .filter(|_| old_value != new_value || starts_new_record(structural))
            {
                visit(field, if new_value.is_some() { Present } else { Absent })?;
            }
        }
        (
            AspectShape::Struct(shape),
            CanonicalAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => match operation {
            AuthoritativePatchDeltaOperation::FieldLevelPatch { patch } => {
                for (field, _) in patch.field_sets() {
                    visit(field, Present)?;
                }
                for field in patch.field_clears() {
                    visit(field, Absent)?;
                }
            }
            AuthoritativePatchDeltaOperation::WholeAspectClear { .. } => {
                for field in shape.fields() {
                    visit(field.key(), Absent)?;
                }
            }
            AuthoritativePatchDeltaOperation::WholeAspectSet { value } => {
                if let ContractValidatedAspectValueView::Struct(value) = value.view() {
                    for field in shape.fields() {
                        visit(
                            field.key(),
                            if value.get(field.key()).is_some() {
                                Present
                            } else {
                                Absent
                            },
                        )?;
                    }
                }
            }
        },
        (
            AspectShape::Scalar(_),
            CanonicalAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => {
            if let Some(field) = bound_field(binding) {
                visit(
                    field,
                    match operation {
                        AuthoritativePatchDeltaOperation::WholeAspectClear { .. } => Absent,
                        AuthoritativePatchDeltaOperation::WholeAspectSet { .. }
                        | AuthoritativePatchDeltaOperation::FieldLevelPatch { .. } => Present,
                    },
                )?;
            }
        }
        _ => {}
    }
    Ok(())
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
