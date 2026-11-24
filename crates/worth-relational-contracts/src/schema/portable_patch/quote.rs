//! Quote concrete portable readmission storage with the original aspect contracts.
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::schema::data::LoweredAspectContractPlan;
use worth_foundational::facade::{
    ContractValidatedAspectValue, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

pub fn add(total: &mut u64, bytes: u64) -> Result<(), Cause> {
    *total = total
        .checked_add(bytes)
        .ok_or(Cause::ChargedBytesOverflow)?;
    Ok(())
}
pub fn nested(total: &mut u64, bytes: usize) -> Result<(), Cause> {
    add(
        total,
        u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?,
    )
}

pub fn readmission_bytes(
    prior_bytes: u64,
    candidate: &PortableRecordAspectPatch,
    plan: Option<&LoweredAspectContractPlan>,
) -> Result<u64, Cause> {
    let bounds = candidate
        .readmission_storage_bounds()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let mut bytes = prior_bytes;
    nested(&mut bytes, bounds.retained_bytes)?;
    nested(&mut bytes, bounds.transient_bytes)?;
    let mut lookup_peak = 0;
    for operation in candidate.operations() {
        // Quote the existing lookup without cloning or introducing a refusal.
        // Missing/mismatched declarations remain the real readmitter's decision.
        let contract = plan
            .and_then(|plan| {
                plan.executable_bindings
                    .iter()
                    .find(|binding| binding.contract.key() == operation.basis().key())
            })
            .map(|binding| &binding.contract);
        if let Some(contract) = contract {
            let owned = contract
                .clone_allocation_bytes()
                .ok_or(Cause::ChargedBytesOverflow)?;
            lookup_peak = lookup_peak.max(owned);
            // The lookup shares its contract; the independently retained patch
            // still keeps that definition alive after the lookup/source retires.
            nested(&mut bytes, contract.owned_allocation_capacity_bytes())?;
            if matches!(operation, PortableAspectPatchOperation::SetWhole { .. }) {
                nested(
                    &mut bytes,
                    ContractValidatedAspectValue::admission_storage_bytes(contract)
                        .ok_or(Cause::ChargedBytesOverflow)?,
                )?;
            }
        }
        if let PortableAspectPatchOperation::PatchFields {
            selected_fields, ..
        } = operation
        {
            for key in selected_fields {
                nested(&mut bytes, key.as_str().len())?;
            }
        }
    }
    nested(&mut bytes, lookup_peak)?;
    Ok(bytes)
}
