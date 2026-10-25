//! Whole-struct effects use the validated candidate's actual two retained roots.
use super::{collection::TouchCollection, *};
use crate::storage::overlay::{PartitionAccess, WorkingState};
use crate::transactions::data::EntityMutationIntent;
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectShape, ContractValidatedAspectValueView,
    ContractValidationInput, PortableAspectPatchOperation as Op, StructAspectValue,
};

pub(super) fn project<'p>(
    proposal: &'p ValidatedRelationalProposal,
    intent: &'p EntityMutationIntent,
    sink: &mut TouchCollection<'p, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    let EntityMutationIntent::ApplyAspectPatch(spec) = intent else {
        unreachable!()
    };
    work.owner_state_lookups += 1;
    let kind = proposal
        .prepared
        .working_state
        .get_partition(spec.entity_id.partition_id)
        .and_then(|p| p.entity_arena.get(&spec.entity_id))
        .and_then(|r| r.kind_id())
        .ok_or(ValidatedMutationTouchProjectionError::MissingEntityKind(
            spec.entity_id,
        ))?;
    for operation in spec.aspect_patch.operations() {
        (sink.checkpoint)(1)?;
        if let Op::PatchFields {
            basis,
            selected_fields,
            ..
        } = operation
        {
            for field in selected_fields {
                sink.selected_field(kind, basis, field);
                if let Some(error) = sink.failure.take() {
                    return Err(error);
                }
            }
            continue;
        }
        let basis = operation.basis();
        if let Some(plan) = proposal.prepared.schema_authority.entity_aspect_plan(kind) {
            (sink.checkpoint)(plan.executable_bindings.len() as u64)?;
        }
        let contract = proposal
            .prepared
            .schema_authority
            .entity_aspect_plan(kind)
            .filter(|p| p.kind_id == kind)
            .and_then(|p| {
                let mut bindings = p
                    .executable_bindings
                    .iter()
                    .filter(|b| b.contract.key() == basis.key());
                let b = bindings.next()?;
                (bindings.next().is_none()
                    && matches!(b.target, AspectBinding::EntityField { .. })
                    && matches!(b.contract.shape(), AspectShape::Struct(_))
                    && b.contract.identity() == basis.identity()
                    && b.contract.revision() == basis.revision())
                .then_some(&b.contract)
            });
        let Some(contract) = contract else {
            sink.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
            continue;
        };
        (sink.checkpoint)(contract.semantic_byte_width() as u64)?;
        work.owner_state_lookups += 2;
        let before = aspect(&proposal.prepared.working_state, spec.entity_id, contract);
        let after = aspect(proposal.proposed_mutation.state(), spec.entity_id, contract);
        let (Ok(before), Ok(after)) = (before, after) else {
            sink.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
            continue;
        };
        let supplied = match operation {
            Op::SetWhole {
                value: ContractValidationInput::Struct(value),
                ..
            } => Some(value),
            Op::ClearWhole { .. } => None,
            _ => {
                sink.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
                continue;
            }
        };
        // All prior fields plus every supplied field, including equal rewrites. Admission still rejects duplicate entity intents.
        let fields = before
            .into_iter()
            .flat_map(StructAspectValue::fields)
            .chain(supplied.into_iter().flat_map(StructAspectValue::fields));
        sink.whole_field(ValidatedEntityStructTouch {
            kind,
            entity: spec.entity_id,
            contract,
            field: None,
            before_present: before.is_some(),
            after_present: after.is_some(),
        });
        for (field, _) in fields {
            if let Some(error) = sink.failure.take() {
                return Err(error);
            }
            sink.whole_field(ValidatedEntityStructTouch {
                kind,
                contract,
                entity: spec.entity_id,
                field: Some(field),
                before_present: before.is_some(),
                after_present: after.is_some(),
            });
        }
    }
    Ok(())
}
fn aspect<'a>(
    state: &'a WorkingState,
    entity: EntityId,
    contract: &AspectContract,
) -> Result<Option<&'a StructAspectValue>, ()> {
    let partition = state.get_partition(entity.partition_id).ok_or(())?;
    let record = partition.entity_arena.get(&entity).ok_or(())?;
    let _ = record; // exact identity/generation was checked by get above
    let extra = partition
        .entity_arena
        .extra_at(crate::storage::substrate::slot_of::<
            crate::storage::substrate::EntityRecordKind,
        >(&entity))
        .ok_or(())?;
    let value = extra
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(contract.key()));
    match value {
        None => Ok(None),
        Some(value) if value.contract() == contract => match value.view() {
            ContractValidatedAspectValueView::Struct(value) => Ok(Some(value)),
            _ => Err(()),
        },
        _ => Err(()),
    }
}
