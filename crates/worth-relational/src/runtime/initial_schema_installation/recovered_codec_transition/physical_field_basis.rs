//! Compare executable field meaning without relaxing physical carrier identity.
use crate::identity::data::KindId;
use crate::schema::data::{DeclaredAspectContractBinding, RelationalSchemaRegistry};
use worth_foundational::facade::{AspectBinding, AspectContract, AspectKey, AspectShape, FieldKey};

#[derive(Debug, Clone)]
pub(crate) struct CodecFieldLocus {
    pub(crate) kind_id: KindId,
    pub(crate) aspect_key: AspectKey,
    pub(crate) field_key: FieldKey,
    pub(crate) binding: AspectBinding,
}

pub(crate) struct OwnedCodecFieldChanges {
    loci: Vec<CodecFieldLocus>,
    _memory: worth_execution::ExecutionMemoryReservation,
}
impl std::ops::Deref for OwnedCodecFieldChanges {
    type Target = [CodecFieldLocus];
    fn deref(&self) -> &Self::Target {
        &self.loci
    }
}

/// This comparison grants no retirement permission. Actual present records
/// separately require complete native producer-publication provenance.
pub(crate) fn codec_field_changes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
) -> Result<OwnedCodecFieldChanges, crate::runtime::RelationalSchemaTransitionAdmissionDenial> {
    let compared=execution.in_scope(|lease| {
        worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
            .run_with_checkpoint(lease,None,|context| {
                let (capacity,bytes)=change_storage_quote(source,context)?;
                let memory=worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease,bytes)
                    .map_err(|stop| ComparisonStop::Request(crate::execution::denial::lease_denial(stop)))?;
                let mut changes=Vec::with_capacity(capacity);
                compare_registries(source,target,context,&mut changes)?;
                Ok(OwnedCodecFieldChanges {loci:changes,_memory:memory})
            })
    }).map_err(|stop| crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(stop.into()))
        .and_then(|result| result.map_err(|stop| crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(stop.into())))
        .and_then(|result| result.0.map_err(|stop| match stop {
            ComparisonStop::Request(cause) => crate::runtime::RelationalSchemaTransitionAdmissionDenial::request(cause),
            ComparisonStop::Contract(detail) => crate::runtime::RelationalSchemaTransitionAdmissionDenial::new(crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch,detail),
        }));
    crate::execution::check_request(execution)
        .map_err(crate::runtime::RelationalSchemaTransitionAdmissionDenial::request)?;
    compared
}

fn compare_registries(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    changes: &mut Vec<CodecFieldLocus>,
) -> Result<(), ComparisonStop> {
    if source.entity_kinds.len() != target.entity_kinds.len()
        || source.relation_kinds.len() != target.relation_kinds.len()
    {
        return Err("codec transition changed the physical kind roster".into());
    }
    for (kind_id, previous) in &source.entity_kinds {
        charge(context, 1)?;
        let next = target
            .entity_kinds
            .get(kind_id)
            .ok_or("codec transition removed an entity kind")?;
        charge_declarations(
            context,
            &previous.aspect_contract_declarations.aspects,
            &next.aspect_contract_declarations.aspects,
        )?;
        if previous.aspect_contract_declarations.aspects.len()
            != next.aspect_contract_declarations.aspects.len()
        {
            return Err("codec transition changed the physical aspect roster".into());
        }
        for (old, new) in previous
            .aspect_contract_declarations
            .aspects
            .iter()
            .zip(&next.aspect_contract_declarations.aspects)
        {
            compare_binding(*kind_id, old, new, changes)?;
        }
        if previous.kind_id != next.kind_id
            || previous.kind_name != next.kind_name
            || previous.aspect_contract_declarations.identity_declarations
                != next.aspect_contract_declarations.identity_declarations
            || previous
                .aspect_contract_declarations
                .merge_policy_declarations
                != next.aspect_contract_declarations.merge_policy_declarations
        {
            return Err("codec transition changed a protected entity contract".into());
        }
    }
    for (kind_id, previous) in &source.relation_kinds {
        charge(context, 1)?;
        let next = target
            .relation_kinds
            .get(kind_id)
            .ok_or("codec transition removed a relation kind")?;
        charge_declarations(
            context,
            &previous.aspect_contract_declarations.aspects,
            &next.aspect_contract_declarations.aspects,
        )?;
        if previous.kind_id != next.kind_id
            || previous.kind_name != next.kind_name
            || previous.cross_context_policy != next.cross_context_policy
            || previous.cascade_delete_policy != next.cascade_delete_policy
            || previous.aspect_contract_declarations != next.aspect_contract_declarations
            || previous.relation_integrity != next.relation_integrity
        {
            return Err("relation codec or integrity transitions are unsupported".into());
        }
    }
    Ok(())
}

fn change_storage_quote(
    source: &RelationalSchemaRegistry,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(usize, u64), ComparisonStop> {
    let mut fields = 0_usize;
    let mut bytes = 0_usize;
    for registration in source.entity_kinds.values() {
        charge(context, 1)?;
        for binding in &registration.aspect_contract_declarations.aspects {
            charge(context, 1)?;
            let AspectShape::Struct(shape) = binding.contract.shape() else {
                continue;
            };
            for field in shape.fields() {
                charge(context, 1)?;
                fields = fields.checked_add(1).ok_or_else(memory_overflow)?;
                bytes = bytes
                    .checked_add(std::mem::size_of::<CodecFieldLocus>())
                    .and_then(|bytes| {
                        bytes.checked_add(binding.contract.key().owned_allocation_capacity_bytes())
                    })
                    .and_then(|bytes| {
                        bytes.checked_add(field.key().owned_allocation_capacity_bytes())
                    })
                    .and_then(|bytes| {
                        bytes.checked_add(binding.binding.owned_allocation_capacity_bytes())
                    })
                    .ok_or_else(memory_overflow)?;
            }
        }
    }
    Ok((fields, u64::try_from(bytes).map_err(|_| memory_overflow())?))
}

fn memory_overflow() -> ComparisonStop {
    ComparisonStop::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}

fn compare_binding(
    kind_id: KindId,
    previous: &DeclaredAspectContractBinding,
    next: &DeclaredAspectContractBinding,
    changes: &mut Vec<CodecFieldLocus>,
) -> Result<(), ComparisonStop> {
    if previous == next {
        return Ok(());
    }
    let old = &previous.contract;
    let new = &next.contract;
    if previous.binding != next.binding || !same_contract_except_shape_revision(old, new) {
        return Err("codec transition changed an aspect binding or protected contract".into());
    }
    let (AspectShape::Struct(old_shape), AspectShape::Struct(new_shape)) =
        (old.shape(), new.shape())
    else {
        return Err("codec transition requires the same struct field carrier".into());
    };
    if old_shape.fields().len() != new_shape.fields().len() {
        return Err("codec transition changed the physical field roster".into());
    }
    let before = changes.len();
    for (old_field, new_field) in old_shape.fields().iter().zip(new_shape.fields()) {
        if old_field.key() != new_field.key()
            || old_field.value_type() != new_field.value_type()
            || old_field.requirement() != new_field.requirement()
            || old_field.absence() != new_field.absence()
            || old_field.evolution() != new_field.evolution()
        {
            return Err("codec transition changed a protected physical field contract".into());
        }
        if old_field.codec() == new_field.codec() {
            continue;
        }
        if old_field.codec().is_none() || new_field.codec().is_none() {
            return Err("codec transition has an unclassified field".into());
        }
        if new.revision() <= old.revision() {
            return Err(
                "codec meaning changed without an advanced semantic aspect revision".into(),
            );
        }
        changes.push(CodecFieldLocus {
            kind_id,
            aspect_key: old.key().clone(),
            field_key: old_field.key().clone(),
            binding: previous.binding.clone(),
        });
    }
    if changes.len() == before {
        return Err("codec transition changed aspect revision without codec meaning".into());
    }
    Ok(())
}

enum ComparisonStop {
    Request(crate::execution::RelationalExecutionDenialCause),
    Contract(String),
}
impl From<&str> for ComparisonStop {
    fn from(detail: &str) -> Self {
        Self::Contract(detail.into())
    }
}
impl From<String> for ComparisonStop {
    fn from(detail: String) -> Self {
        Self::Contract(detail)
    }
}

fn charge_declarations(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    previous: &[DeclaredAspectContractBinding],
    next: &[DeclaredAspectContractBinding],
) -> Result<(), ComparisonStop> {
    for binding in previous.iter().chain(next) {
        let width = binding
            .contract
            .semantic_byte_width()
            .checked_add(binding.binding.owned_allocation_capacity_bytes())
            .ok_or(ComparisonStop::Request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            ))?;
        charge(context, width)?;
    }
    Ok(())
}
fn charge(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), ComparisonStop> {
    context
        .checkpoint(u64::try_from(work).map_err(|_| {
            ComparisonStop::Request(
                crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?)
        .map_err(|stop| ComparisonStop::Request(stop.into()))
}

fn same_contract_except_shape_revision(old: &AspectContract, new: &AspectContract) -> bool {
    old.key() == new.key()
        && old.identity() == new.identity()
        && old.masks() == new.masks()
        && old.absence() == new.absence()
        && old.equivalence() == new.equivalence()
        && old.evolution() == new.evolution()
}
