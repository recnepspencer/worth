use super::{array_charge, btree_charge, sum, BridgeRetentionDenial as D};
use worth_foundational::facade::{
    AspectBinding, AspectContract, AspectMask, AspectShape, AspectValue, CanonicalFieldPath,
    ContractValidatedAspectArtifact, ContractValidatedAspectValueView, FieldDeclaration, FieldKey,
    InternedString, ProjectionMask,
};

pub(in crate::conditional_execution) fn visit(work: &mut usize) -> Result<(), D> {
    *work = work.checked_sub(1).ok_or(D::PreparationExhausted)?;
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
    work: &mut usize,
) -> Result<u64, D> {
    visit(work)?;
    let mut size = bytes(contract.key().as_str().len())?;
    if let AspectShape::Struct(shape) = contract.shape() {
        size = sum(&[
            size,
            array_charge::<FieldDeclaration>(shape.fields().len())?,
        ])?;
        for field in shape.fields() {
            visit(work)?;
            size = sum(&[size, bytes(field.key().as_str().len())?])?;
        }
    }
    Ok(size)
}

pub(in crate::conditional_execution) fn path(
    path: &CanonicalFieldPath,
    work: &mut usize,
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
    work: &mut usize,
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
    work: &mut usize,
) -> Result<u64, D> {
    visit(work)?;
    let payload = artifact.payload();
    let mut size = sum(&[
        bytes(payload.key().as_str().len())?,
        contract(payload.contract(), work)?,
    ])?;
    match payload.view() {
        ContractValidatedAspectValueView::Scalar(value) => {
            size = sum(&[size, scalar(value)?])?;
        }
        ContractValidatedAspectValueView::Struct(fields) => {
            let mut count = 0usize;
            for (key, value) in fields.fields() {
                visit(work)?;
                count = count.checked_add(1).ok_or(D::BytesExhausted)?;
                size = sum(&[size, bytes(key.as_str().len())?, scalar(value)?])?;
            }
            size = sum(&[size, btree_charge::<FieldKey, AspectValue>(count)?])?;
        }
    }
    Ok(size)
}

// Clone materializes length-sized String buffers. The source's spare capacity
// is neither copied nor owned by the retained observation/trigger.
fn scalar(value: &AspectValue) -> Result<u64, D> {
    match value {
        AspectValue::Decimal(value) => bytes(value.0.len()),
        AspectValue::BigInt(value) => bytes(value.0.len()),
        AspectValue::Rational(value) => sum(&[
            bytes(value.numerator.0.len())?,
            bytes(value.denominator.0.len())?,
        ]),
        AspectValue::String(InternedString::Raw(value)) => bytes(value.len()),
        _ => Ok(0),
    }
}

pub(in crate::conditional_execution) fn binding(value: &AspectBinding) -> Result<u64, D> {
    match value {
        AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
            bytes(field.as_str().len())
        }
        _ => Ok(0),
    }
}
