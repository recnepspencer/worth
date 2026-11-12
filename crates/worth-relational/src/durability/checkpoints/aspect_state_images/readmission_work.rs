//! Work of Foundation readmission, separate from conservative backing bounds.
use super::*;
use crate::durability::checkpoints::images::ImageReadmission;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_foundational::facade::{
    AspectShape, ContractValidatedAspectArtifact, ContractValidatedAspectValue,
    ContractValidationInput, PortableAspectContractBasis,
};

fn product(a: u64, b: u64) -> Result<u64, Stop> {
    a.checked_mul(b)
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}
fn add(a: u64, b: u64) -> Result<u64, Stop> {
    a.checked_add(b)
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}

pub(super) fn state(
    state: Option<&PortableRecordAspectState>,
    contracts: &CheckpointAspectContractCatalog,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<(), Stop> {
    let Some(state) = state else {
        return Ok(());
    };
    let count = state.entries().len() as u64;
    let mut longest_contract_key = 0;
    for (key, _, _) in contracts.contracts.keys() {
        admission.charge(1)?;
        longest_contract_key = longest_contract_key.max(key.as_str().len() as u64);
    }
    let mut longest_key = 0;
    for entry in state.entries() {
        admission.charge(1)?;
        longest_key = longest_key.max(entry.basis().key().as_str().len() as u64);
    }
    let comparisons = 11
        * (count
            .checked_add(1)
            .ok_or(Cause::ChargedBytesOverflow)?
            .ilog2() as u64
            + 1);
    for entry in state.entries() {
        let key = entry.basis().key().as_str().len() as u64;
        // Two exact-query temporaries, two canonical-map keys, and a denial key.
        admission.charge(product(key, 5)?)?;
        // Validation artifact, admission Vec (including its geometric moves),
        // ordered map entry, final Vec and boxed-slice transfer.
        admission.charge(add(
            size_of::<ContractValidatedAspectArtifact>() as u64,
            product(size_of::<ContractValidatedAspectValue>() as u64, 6)?,
        )?)?;
        admission.charge(product(comparisons, add(1, add(key, longest_key)?)?)?)?;
        let Some(contract) =
            matching_contract(contracts, entry.basis(), longest_contract_key, admission)?
        else {
            continue; // Foundation retains its precise missing-contract refusal.
        };
        // Catalog return and admitted value each clone one immutable Arc.
        // Reachable definition retention is separately reserved by the holder;
        // it is not newly copied payload work.
        admission.charge(2)?;
        match entry.value() {
            ContractValidationInput::Scalar(value) => {
                admission.charge(value.semantic_byte_width() as u64)?
            }
            ContractValidationInput::Struct(value) => {
                let mut longest_field = 0;
                for (key, scalar) in value.fields() {
                    admission.charge(add(1, scalar.semantic_byte_width() as u64)?)?;
                    longest_field = longest_field.max(key.as_str().len() as u64);
                }
                if let AspectShape::Struct(shape) = contract.shape() {
                    let search = (value
                        .len()
                        .checked_add(1)
                        .ok_or(Cause::ChargedBytesOverflow)?
                        .ilog2()
                        + 1) as u64;
                    for field in shape.fields() {
                        let comparison =
                            add(1, add(field.key().as_str().len() as u64, longest_field)?)?;
                        // Required-field binary search and each supplied field's
                        // linear declaration lookup, before Foundation executes.
                        admission.charge(product(add(search, value.len() as u64)?, comparison)?)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn matching_contract<'a>(
    contracts: &'a CheckpointAspectContractCatalog,
    basis: &PortableAspectContractBasis,
    longest_key: u64,
    admission: &ImageReadmission<'_, '_, '_>,
) -> Result<Option<&'a AspectContract>, Stop> {
    // Foundation will separately execute its ordered exact lookup. This bound
    // funds that traversal in addition to the preflight's own borrowed scan.
    let levels = (contracts
        .contracts
        .len()
        .checked_add(1)
        .ok_or(Cause::ChargedBytesOverflow)?
        .ilog2()
        + 1) as u64;
    admission.charge(product(
        22 * levels,
        add(1, add(longest_key, basis.key().as_str().len() as u64)?)?,
    )?)?;
    // The caller's existing scratch reservation is live before this clone.
    let query = (basis.key().clone(), basis.identity(), basis.revision());
    let selected = contracts.contracts.get(&query);
    drop(query);
    if selected.is_some() {
        return Ok(selected);
    }
    // Match the production trait's reverse-key fallback even for corrupt input;
    // identity/revision refusal still belongs to Foundation.
    for ((key, _, _), contract) in contracts.contracts.iter().rev() {
        admission.charge(product(
            2,
            add(
                1,
                add(key.as_str().len() as u64, basis.key().as_str().len() as u64)?,
            )?,
        )?)?;
        if key == basis.key() {
            return Ok(Some(contract));
        }
    }
    Ok(None)
}
