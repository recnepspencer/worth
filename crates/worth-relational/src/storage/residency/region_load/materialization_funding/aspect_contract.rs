//! Memory and work accounting for a cloned retained aspect contract.
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_foundational::facade::AspectContract;

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
        AspectShape::Scalar(_) | AspectShape::Opaque(_) | AspectShape::Reference(_) => {}
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
