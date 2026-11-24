//! Foundation-owned state roots, admitted cells and construction excess.
use super::super::*;
use worth_foundational::facade::{
    CanonicalAspectStateMap, ContractValidatedAspectArtifact, ContractValidatedAspectValue,
    PortableRecordAspectStateEntry,
};

fn checked(bytes: Option<usize>) -> Result<u64> {
    bytes
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(overflow)
}

pub(super) fn state_roots(count: usize, maximum_key_width: usize, contract: u64) -> Result<Quote> {
    let roots = checked(CanonicalAspectStateMap::construction_storage_bytes(count))?;
    let cells = times(
        checked(ContractValidatedAspectValue::admission_cell_storage_bytes())?,
        count,
    )?;
    let scratch = checked(CanonicalAspectStateMap::construction_scratch_bytes(
        count,
        maximum_key_width,
    ))?;
    // The original wire candidate/validation rosters and all tree construction
    // excess coexist with the separately reserved independent native holder.
    let wire = sum(
        vector::<PortableRecordAspectStateEntry>(count)?,
        sum(vector::<ContractValidatedAspectArtifact>(count)?, scratch)?,
    )?;
    let native = sum(sum(roots, cells)?, times(contract, count)?)?;
    Ok(Quote::bytes(wire, native))
}
