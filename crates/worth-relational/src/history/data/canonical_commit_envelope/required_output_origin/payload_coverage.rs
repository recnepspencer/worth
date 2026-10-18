use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{EntityId, RelationId};
use crate::publication::patch::data::{
    PublishedAuthoritativePatchOperation as Operation, PublishedAuthoritativePatchValue as Value,
    RecordStructuralChange,
};
use crate::transactions::data::RecordRef;
use worth_execution::MapKernelContext;
use worth_foundational::facade::{
    AspectBinding, AuthoritativeRecordAspectState, ContractValidatedAspectValue,
    ContractValidatedAspectValueView, StructAspectValue,
};

#[derive(Debug)]
pub(super) enum OutputPayloadCoverageFailure {
    Request(crate::execution::RelationalExecutionDenialCause),
    Unsupported,
    Corrupt(String),
}
impl From<&str> for OutputPayloadCoverageFailure {
    fn from(value: &str) -> Self {
        Self::Corrupt(value.into())
    }
}

/// Every native key/value comparison is admitted before it runs. This proof
/// walks present aspects against actual operations, then exact present fields.
pub(super) fn validate_created_output_payload(
    envelope: &CanonicalCommitEnvelope,
    id: EntityId,
    payload: &AuthoritativeRecordAspectState,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), OutputPayloadCoverageFailure> {
    validate_created_payload(envelope, &RecordRef::Entity(id), Some(payload), context)
}

pub(super) fn validate_created_relation_payload(
    envelope: &CanonicalCommitEnvelope,
    id: RelationId,
    payload: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), OutputPayloadCoverageFailure> {
    charge(context, envelope.record_allocations.len())?;
    if envelope
        .record_allocations
        .iter()
        .filter(|allocation| allocation.record() == &RecordRef::Relation(id))
        .count()
        != 1
    {
        return Err("required relation has no unique native allocation".into());
    }
    validate_created_payload(envelope, &RecordRef::Relation(id), payload, context)
}

fn validate_created_payload(
    envelope: &CanonicalCommitEnvelope,
    target: &RecordRef,
    payload: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), OutputPayloadCoverageFailure> {
    charge(context, envelope.patch.authoritative_record_patches.len())?;
    let mut records = envelope
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|record| &record.target == target);
    let record = records
        .next()
        .ok_or("required output has no native create publication")?;
    if records.next().is_some() || record.structural_change != RecordStructuralChange::Created {
        return Err("required output lacks one complete native create publication".into());
    }
    if record.contains_opaque_aspect {
        return Err(OutputPayloadCoverageFailure::Unsupported);
    }
    let operations = record.authoritative_patch.full_grammar_operations();
    let Some(payload) = payload.filter(|payload| !payload.aspects().is_empty()) else {
        return if matches!(target, RecordRef::Relation(_)) && operations.is_empty() {
            Ok(())
        } else {
            Err(OutputPayloadCoverageFailure::Unsupported)
        };
    };
    for (_, present) in payload.aspects().entries() {
        let mut selected = None;
        for operation in operations {
            charge(
                context,
                operation
                    .aspect_key()
                    .as_str()
                    .len()
                    .saturating_add(present.key().as_str().len())
                    .saturating_add(1),
            )?;
            if operation.aspect_key() == present.key() {
                if selected.is_some() {
                    return Err(OutputPayloadCoverageFailure::Unsupported);
                }
                selected = Some(operation);
            }
        }
        covered_value(
            present,
            selected.ok_or("required output present cell has no actual native operation")?,
            target,
            context,
        )?;
    }
    Ok(())
}

fn covered_value(
    present: &ContractValidatedAspectValue,
    operation: &Operation,
    target: &RecordRef,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), OutputPayloadCoverageFailure> {
    charge(context, 1)?;
    let (identity, revision, binding) = match operation {
        Operation::WholeAspectSet {
            aspect_identity,
            contract_revision,
            binding,
            ..
        }
        | Operation::WholeAspectClear {
            aspect_identity,
            contract_revision,
            binding,
            ..
        }
        | Operation::FieldLevelPatch {
            aspect_identity,
            contract_revision,
            binding,
            ..
        } => (*aspect_identity, *contract_revision, binding),
    };
    if identity != present.contract_identity() || revision != present.contract_revision() {
        return Err("required output operation and present contract identity differ".into());
    }
    if !matches!(
        (target, binding),
        (RecordRef::Entity(_), AspectBinding::EntityField { .. })
            | (RecordRef::Relation(_), AspectBinding::RelationField { .. })
            | (
                RecordRef::Relation(_),
                AspectBinding::RelationSourceEndpoint
            )
            | (
                RecordRef::Relation(_),
                AspectBinding::RelationTargetEndpoint
            )
    ) {
        return Err(OutputPayloadCoverageFailure::Unsupported);
    }
    let equal = match (operation, present.view()) {
        (
            Operation::WholeAspectSet {
                value: Value::Scalar(expected),
                ..
            },
            ContractValidatedAspectValueView::Scalar(actual),
        ) => {
            charge(
                context,
                expected
                    .semantic_byte_width()
                    .saturating_add(actual.semantic_byte_width()),
            )?;
            expected == actual
        }
        (
            Operation::WholeAspectSet {
                value: Value::Struct(expected),
                ..
            },
            ContractValidatedAspectValueView::Struct(actual),
        ) => {
            charge_struct(context, expected)?;
            charge_struct(context, actual)?;
            expected == actual
        }
        (
            Operation::FieldLevelPatch {
                field_sets,
                field_clears,
                ..
            },
            ContractValidatedAspectValueView::Struct(actual),
        ) if field_clears.is_empty() => {
            if actual.len() != field_sets.len() {
                return Err(
                    "required output field operation roster differs from present payload".into(),
                );
            }
            for (key, value) in actual.fields() {
                let mut matched = false;
                for set in field_sets {
                    charge(
                        context,
                        key.as_str()
                            .len()
                            .saturating_add(set.field.as_str().len())
                            .saturating_add(1),
                    )?;
                    if &set.field == key {
                        charge(
                            context,
                            value
                                .semantic_byte_width()
                                .saturating_add(set.value.semantic_byte_width()),
                        )?;
                        if &set.value != value {
                            return Err(
                                "required output field differs from its native operation".into()
                            );
                        }
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return Err("required output present field has no native set operation".into());
                }
            }
            true
        }
        _ => return Err(OutputPayloadCoverageFailure::Unsupported),
    };
    if equal {
        Ok(())
    } else {
        Err("required output present payload differs from its native operation".into())
    }
}

fn charge_struct(
    context: &mut MapKernelContext<'_, '_>,
    value: &StructAspectValue,
) -> Result<(), OutputPayloadCoverageFailure> {
    for (field, value) in value.fields() {
        charge(
            context,
            field
                .as_str()
                .len()
                .saturating_add(value.semantic_byte_width())
                .saturating_add(1),
        )?;
    }
    Ok(())
}

pub(super) fn charge(
    context: &mut MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), OutputPayloadCoverageFailure> {
    let units = u64::try_from(work).map_err(|_| {
        OutputPayloadCoverageFailure::Request(
            crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
        )
    })?;
    context
        .checkpoint(units)
        .map_err(|cause| OutputPayloadCoverageFailure::Request(cause.into()))
}
