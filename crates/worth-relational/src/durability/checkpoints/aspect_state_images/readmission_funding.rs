//! Readmission admits actual state backing before validation and boxing.
use super::*;
use crate::durability::checkpoints::images::ImageReadmission;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::{ContractValidatedAspectArtifact, ContractValidatedAspectValue};

fn add(a: u64, b: u64) -> Result<u64, Stop> {
    a.checked_add(b)
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}
fn array<T>(count: usize) -> Result<u64, Stop> {
    std::alloc::Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}
pub(in crate::durability::checkpoints) fn readmit_state_using(
    state: Option<PortableRecordAspectState>,
    contracts: &CheckpointAspectContractCatalog,
    owner_bytes: u64,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<
    (
        Option<AuthoritativeRecordAspectState>,
        Option<ExecutionMemoryReservation>,
    ),
    Stop,
> {
    let mut retained = owner_bytes;
    let mut scratch = 0;
    if let Some(state) = &state {
        let count = state.entries().len();
        retained = add(retained, array::<ContractValidatedAspectValue>(count)?)?;
        scratch = add(scratch, array::<ContractValidatedAspectArtifact>(count)?)?;
        let capacity = count
            .checked_mul(2)
            .ok_or(Cause::ChargedBytesOverflow)?
            .max(4);
        scratch = add(scratch, array::<ContractValidatedAspectValue>(capacity)?)?;
        let node = std::alloc::Layout::new::<(
            usize,
            u16,
            u16,
            [AspectKey; 11],
            [ContractValidatedAspectValue; 11],
            [usize; 12],
        )>()
        .size() as u64;
        if count != 0 {
            scratch = add(
                scratch,
                node.checked_mul((count.div_ceil(5) + 2) as u64)
                    .ok_or(Cause::ChargedBytesOverflow)?,
            )?;
        }
        for entry in state.entries() {
            admission.charge(1)?;
            let key = entry.basis().owned_allocation_capacity_bytes() as u64;
            let value = input_bytes(entry.value(), admission)?;
            retained = add(retained, add(add(key, value)?, admission.contract_bound)?)?;
            scratch = add(
                scratch,
                add(
                    key.checked_mul(4).ok_or(Cause::ChargedBytesOverflow)?,
                    admission.contract_bound,
                )?,
            )?;
        }
    }
    admission.charge(add(retained, scratch)?)?;
    let memory = admission.reserve(retained)?;
    let _scratch = admission.reserve(scratch)?;
    Ok((readmit_state(state, contracts)?, memory))
}

pub(crate) fn contract_clone_bytes(
    contract: &AspectContract,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<u64, Cause> {
    use worth_foundational::facade::{AspectShape, FieldDeclaration};
    charge_contract_work(request, 1)?;
    let mut bytes = contract.key().as_str().len() as u64;
    match contract.shape() {
        AspectShape::Struct(shape) => {
            // The field count is available without walking the declarations.
            // Admit that traversal before touching any field, including when
            // this quote is used only to find a catalog's largest contract.
            charge_contract_work(request, shape.fields().len() as u64)?;
            bytes = bytes
                .checked_add(
                    std::alloc::Layout::array::<FieldDeclaration>(shape.fields().len())
                        .ok()
                        .and_then(|layout| u64::try_from(layout.size()).ok())
                        .ok_or(Cause::ChargedBytesOverflow)?,
                )
                .ok_or(Cause::ChargedBytesOverflow)?;
            for field in shape.fields() {
                request.consult()?;
                bytes = bytes
                    .checked_add(field.key().as_str().len() as u64)
                    .ok_or(Cause::ChargedBytesOverflow)?;
            }
        }
        AspectShape::Scalar(_)
        | AspectShape::Opaque(_)
        | AspectShape::Reference(_)
        | AspectShape::Content => {}
    }
    Ok(bytes)
}

fn charge_contract_work(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    units: u64,
) -> Result<(), Cause> {
    let mut outcome = None;
    let handoff = request.run_native_with_checkpoint(|context| {
        outcome = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (handoff, outcome) {
        (_, Some(Err(cause))) => Err(worth_execution::WorkCeilingDenial::Stopped(cause).into()),
        (Err(cause), _) => Err(cause.into()),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed contract charge executes its callback"),
    }
}

fn input_bytes(
    input: &worth_foundational::facade::ContractValidationInput,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<u64, Stop> {
    use worth_foundational::facade::{AspectValue, ContractValidationInput as Input, FieldKey};
    match input {
        Input::Scalar(value) => Ok(value.owned_allocation_capacity_bytes() as u64),
        Input::Struct(value) => {
            let mut bytes = array::<(FieldKey, AspectValue)>(value.len())?;
            for (key, scalar) in value.fields() {
                admission.charge(1)?;
                bytes = add(
                    bytes,
                    add(
                        key.owned_allocation_capacity_bytes() as u64,
                        scalar.owned_allocation_capacity_bytes() as u64,
                    )?,
                )?;
            }
            Ok(bytes)
        }
    }
}
