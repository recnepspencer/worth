//! Field revision meaning derived from the exact canonical aspect evidence.
use super::borrowed_evidence::{BorrowedAspectDeltaEvidence, BorrowedPatchOperation};
use super::data::{CanonicalAspectDeltaEvidence, EvaluatedAspectBinding};
use crate::storage::data::RelationalFieldPresence;
use worth_foundational::facade::{
    AspectBinding, AspectShape, ContractValidatedAspectValueView, FieldKey,
};

pub(crate) fn changed_fields(
    binding: &EvaluatedAspectBinding,
    structural: crate::publication::patch::data::RecordStructuralChange,
) -> Vec<(FieldKey, RelationalFieldPresence)> {
    let mut fields = Vec::new();
    let result = visit_changed_fields(binding, structural, |field, presence| {
        fields.push((field.clone(), presence));
        Ok::<(), std::convert::Infallible>(())
    });
    if let Err(impossible) = result {
        match impossible {}
    }
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
    let result = visit_field_changes_for_evidence(
        binding,
        shape,
        evidence,
        structural,
        |field, presence| {
            fields.push((field.clone(), presence));
            Ok::<(), std::convert::Infallible>(())
        },
    );
    if let Err(impossible) = result {
        match impossible {}
    }
    fields
}

fn visit_field_changes_for_evidence<E>(
    binding: &AspectBinding,
    shape: &AspectShape,
    evidence: &CanonicalAspectDeltaEvidence,
    structural: crate::publication::patch::data::RecordStructuralChange,
    visit: impl FnMut(&FieldKey, RelationalFieldPresence) -> Result<(), E>,
) -> Result<(), E> {
    visit_borrowed_field_changes(binding, shape, &evidence.borrowed(), structural, visit)
}

pub(crate) fn visit_borrowed_field_changes<E>(
    binding: &AspectBinding,
    shape: &AspectShape,
    evidence: &BorrowedAspectDeltaEvidence<'_>,
    structural: crate::publication::patch::data::RecordStructuralChange,
    mut visit: impl FnMut(&FieldKey, RelationalFieldPresence) -> Result<(), E>,
) -> Result<(), E> {
    use RelationalFieldPresence::{Absent, Present};
    match (shape, evidence) {
        (
            AspectShape::Struct(shape),
            BorrowedAspectDeltaEvidence::StructAspectValueTransition {
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
            BorrowedAspectDeltaEvidence::ScalarAspectValueTransition {
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
            BorrowedAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => match operation {
            BorrowedPatchOperation::FieldLevelPatch { patch } => {
                for (field, _) in patch.field_sets() {
                    visit(field, Present)?;
                }
                for field in patch.field_clears() {
                    visit(field, Absent)?;
                }
            }
            BorrowedPatchOperation::WholeAspectClear { .. } => {
                for field in shape.fields() {
                    visit(field.key(), Absent)?;
                }
            }
            BorrowedPatchOperation::WholeAspectSet { value } => {
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
            BorrowedAspectDeltaEvidence::AuthoritativePatch { operation, .. },
        ) => {
            if let Some(field) = bound_field(binding) {
                visit(
                    field,
                    match operation {
                        BorrowedPatchOperation::WholeAspectClear { .. } => Absent,
                        BorrowedPatchOperation::WholeAspectSet { .. }
                        | BorrowedPatchOperation::FieldLevelPatch { .. } => Present,
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
