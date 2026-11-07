use super::{array_charge, sum, BridgeRetentionDenial as D};
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectMask, CanonicalFieldPath, ContractValidatedAspectArtifact,
    FieldKey, ProjectionMask,
};

pub(in crate::conditional_execution) fn visit(work: &mut Option<usize>) -> Result<(), D> {
    if let Some(remaining) = work {
        *remaining = remaining.checked_sub(1).ok_or(D::PreparationExhausted)?;
    }
    Ok(())
}

pub(in crate::conditional_execution) fn bytes(value: usize) -> Result<u64, D> {
    if value == usize::MAX {
        return Err(D::BytesExhausted);
    }
    u64::try_from(value).map_err(|_| D::BytesExhausted)
}

pub(in crate::conditional_execution) fn contract(
    contract: &AspectContract,
    work: &mut Option<usize>,
) -> Result<u64, D> {
    visit(work)?;
    // Definitions are immutable shared backing. Retention owns that backing's
    // cached payload extent, including its actual spare capacity, not a copy.
    bytes(contract.owned_allocation_capacity_bytes())
}

pub(in crate::conditional_execution) fn path(
    path: &CanonicalFieldPath,
    work: &mut Option<usize>,
) -> Result<u64, D> {
    let mut size = array_charge::<FieldKey>(path.fields().len())?;
    for field in path.fields() {
        visit(work)?;
        size = sum(&[size, bytes(field.as_str().len())?])?;
    }
    Ok(size)
}

pub(in crate::conditional_execution) fn mask(
    mask: &AspectMask<ProjectionMask>,
    work: &mut Option<usize>,
) -> Result<u64, D> {
    let mut size = array_charge::<CanonicalFieldPath>(mask.paths().len())?;
    for item in mask.paths() {
        visit(work)?;
        size = sum(&[size, path(item, work)?])?;
    }
    Ok(size)
}

pub(in crate::conditional_execution) fn artifact(
    artifact: &ContractValidatedAspectArtifact,
    work: &mut Option<usize>,
) -> Result<u64, D> {
    visit(work)?;
    // Artifact cloning shares the admitted backing. The cached reachable extent
    // bounds the payload kept alive; this is not a transient clone quote.
    bytes(artifact.payload().owned_allocation_capacity_bytes())
}

pub(in crate::conditional_execution) fn binding(value: &AspectBinding) -> Result<u64, D> {
    match value {
        AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
            bytes(field.as_str().len())
        }
        _ => Ok(0),
    }
}
