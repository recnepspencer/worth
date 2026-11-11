//! Borrowed source ordinals preserve the old maps' ordering and winners.
use super::*;
use crate::schema::data::LoweredAspectContractBinding;
use crate::transactions::data::AspectFieldTargetRejectionReason as Reason;
use worth_foundational::facade::*;
#[derive(Clone, Copy)]
pub(super) struct Selected<'a> {
    pub(super) binding: &'a LoweredAspectContractBinding,
    pub(super) field: &'a FieldKey,
    pub(super) value: &'a AspectValue,
    ordinal: usize,
    pub(super) scalar: bool,
}
pub(super) fn select<'a>(
    fields: &'a AspectFieldPatch,
    plan: &'a LoweredAspectContractPlan,
    domain: FieldAuthoringDomain,
) -> Result<Vec<Selected<'a>>, RecordAspectPatchDenial> {
    let mut rows = Vec::with_capacity(fields.len());
    // Validation order is the original full-locator order, before regrouping.
    for (ordinal, (target, value)) in fields.iter().enumerate() {
        let field = match target.field_path().fields() {
            [field] => field,
            _ => return Err(denied(target, Reason::NestedFieldPath)),
        };
        let binding = plan
            .executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == target.aspect().aspect_key())
            .ok_or_else(|| denied(target, Reason::UndeclaredAspect))?;
        let (scalar, structure) = match domain {
            FieldAuthoringDomain::Entity => (
                binding.targets_entity_scalar_field(field),
                binding.targets_entity_struct_field(field),
            ),
            FieldAuthoringDomain::Relation => (
                binding.targets_relation_scalar_field(field),
                binding.targets_relation_struct_field(field),
            ),
        };
        let scalar = match (binding.contract.shape(), scalar, structure) {
            (AspectShape::Scalar(_), true, _) => true,
            (AspectShape::Struct(_), _, true) => false,
            _ => return Err(denied(target, Reason::FieldPathNotAdmittedByAspectBinding)),
        };
        rows.push(Selected {
            binding,
            field,
            value,
            ordinal,
            scalar,
        });
    }
    rows.sort_unstable_by(|a, b| {
        b.scalar
            .cmp(&a.scalar)
            .then(a.binding.aspect_key().cmp(b.binding.aspect_key()))
            .then(a.field.cmp(b.field))
            .then(a.ordinal.cmp(&b.ordinal))
    });
    let mut read = 0;
    let mut write = 0;
    while read < rows.len() {
        let first = rows[read];
        let mut end = read + 1;
        while end < rows.len()
            && rows[end].scalar == first.scalar
            && rows[end].binding.aspect_key() == first.binding.aspect_key()
            && (first.scalar || rows[end].field == first.field)
        {
            end += 1;
        }
        let last = rows[read..end].iter().max_by_key(|row| row.ordinal).expect(
            /* Proof: end begins at read + 1 in this nonempty run. */
            "selected source run is nonempty",
        );
        rows[write] = Selected {
            value: last.value,
            ordinal: last.ordinal,
            ..first
        };
        write += 1;
        read = end;
    }
    rows.truncate(write);
    Ok(rows)
}
pub(super) fn groups<'a, 'value: 'a>(
    rows: &'a [Selected<'value>],
) -> impl Iterator<Item = &'a [Selected<'value>]> + 'a {
    let mut remaining = rows;
    std::iter::from_fn(move || {
        let first = remaining.first()?;
        let end = remaining.partition_point(|row| {
            row.scalar == first.scalar && row.binding.aspect_key() == first.binding.aspect_key()
        });
        let (group, tail) = remaining.split_at(end);
        remaining = tail;
        Some(group)
    })
}
pub(super) fn build(
    rows: &[Selected<'_>],
    purpose: PortablePatchReadmissionPurpose,
) -> Result<Vec<PortableAspectPatchOperation>, RecordAspectPatchDenial> {
    let mut operations = Vec::with_capacity(groups(rows).count());
    for group in groups(rows) {
        let basis = PortableAspectContractBasis::from_contract(&group[0].binding.contract);
        let operation =
            if group[0].scalar {
                PortableAspectPatchOperation::SetWhole {
                    basis,
                    value: ContractValidationInput::Scalar(group[0].value.clone()),
                }
            } else if purpose == PortablePatchReadmissionPurpose::RecordCreation {
                let mut pairs = Vec::with_capacity(group.len());
                pairs.extend(
                    group
                        .iter()
                        .map(|row| (row.field.clone(), row.value.clone())),
                );
                let value = StructAspectValue::from_sorted_fields(pairs.into_boxed_slice())
                    .map_err(|_| RecordAspectPatchDenial::StructValueConstructionDenied {
                        aspect_key: basis.key().clone(),
                    })?;
                PortableAspectPatchOperation::SetWhole {
                    basis,
                    value: ContractValidationInput::Struct(value),
                }
            } else {
                let mut selected_fields = Vec::with_capacity(group.len());
                let mut field_sets = Vec::with_capacity(group.len());
                for row in group {
                    selected_fields.push(row.field.clone());
                    field_sets.push(PortableAspectFieldSet::new(
                        row.field.clone(),
                        row.value.clone(),
                    ));
                }
                PortableAspectPatchOperation::PatchFields {
                    basis,
                    selected_fields,
                    field_sets,
                    field_clears: Vec::new(),
                }
            };
        operations.push(operation);
    }
    Ok(operations)
}
fn denied(target: &AspectFieldLocator, reason: Reason) -> RecordAspectPatchDenial {
    RecordAspectPatchDenial::FieldAuthoringDenied {
        target: target.clone(),
        reason,
    }
}
