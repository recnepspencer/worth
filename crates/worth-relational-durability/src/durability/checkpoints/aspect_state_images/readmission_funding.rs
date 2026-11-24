//! Readmission admits actual state backing before validation and boxing.
use super::*;
use crate::durability::checkpoints::images::ImageReadmission;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::{ContractValidatedAspectArtifact, ContractValidatedAspectValue};
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause as Cause;

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
pub(in crate::internal_port::durability::checkpoints) fn readmit_state_using(
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
        use worth_foundational::facade::CanonicalAspectStateMap as Map;
        let checked = |bytes: Option<usize>| {
            bytes
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| Stop::from(Cause::ChargedBytesOverflow))
        };
        let count = state.entries().len();
        let maximum_key_width = state
            .entries()
            .iter()
            .map(|entry| entry.basis().key().as_str().len())
            .max()
            .unwrap_or(0);
        retained = add(retained, checked(Map::construction_storage_bytes(count))?)?;
        retained = add(
            retained,
            checked(ContractValidatedAspectValue::admission_cell_storage_bytes())?
                .checked_mul(count as u64)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        scratch = add(scratch, array::<ContractValidatedAspectArtifact>(count)?)?;
        scratch = add(
            scratch,
            checked(Map::construction_scratch_bytes(count, maximum_key_width))?,
        )?;
        for entry in state.entries() {
            admission.charge(1)?;
            let key = u64::try_from(entry.basis().key().as_str().len())
                .map_err(|_| Cause::ChargedBytesOverflow)?;
            let value = input_bytes(entry.value(), admission)?;
            // The exact catalog door clones an owned tuple key. Fund that
            // preflight temporary from this same request before the lookup.
            let basis = entry.basis();
            admission.charge(key)?;
            let lookup_memory = admission.reserve(key)?;
            let contract =
                contracts.exact_contract_for(basis.key(), basis.identity(), basis.revision());
            let contract = contract
                .as_ref()
                .map_or(0, AspectContract::owned_allocation_capacity_bytes);
            drop(lookup_memory);
            // Absent exact definitions retain no output. Foundation still owns
            // the later missing/identity/revision mismatch denial and fallback.
            let contract = u64::try_from(contract).map_err(|_| Cause::ChargedBytesOverflow)?;
            retained = add(
                retained,
                add(
                    add(
                        key.checked_mul(2).ok_or(Cause::ChargedBytesOverflow)?,
                        value,
                    )?,
                    contract,
                )?,
            )?;
            // Entry admission and map insertion each make a temporary owned key.
            scratch = add(
                scratch,
                key.checked_mul(2).ok_or(Cause::ChargedBytesOverflow)?,
            )?;
        }
    }
    // Fixed Extra construction executes even when no aspect state exists.
    admission.charge(owner_bytes)?;
    let memory = admission.reserve(retained)?;
    let _scratch = admission.reserve(scratch)?;
    super::readmission_work::state(state.as_ref(), contracts, admission)?;
    Ok((readmit_state(state, contracts)?, memory))
}

fn input_bytes(
    input: &worth_foundational::facade::ContractValidationInput,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<u64, Stop> {
    admission.charge(1)?;
    u64::try_from(input.owned_allocation_capacity_bytes())
        .map_err(|_| Cause::ChargedBytesOverflow.into())
}
