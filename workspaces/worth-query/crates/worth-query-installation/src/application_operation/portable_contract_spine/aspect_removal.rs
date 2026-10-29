//! Portable removal preserves the installed Optional struct contract in full.
use super::*;
pub(super) fn compile(
    schema: &str,
    native_contracts: &[WorthQueryPortableNativeAspectContractRecord],
    entity: &str,
    aspect: &str,
) -> SpineResult<WorthQueryPortableOperationTouchScope> {
    let native = native_contract(native_contracts, schema, entity, aspect)?;
    if native.contract().absence() != worth_foundational::facade::AbsenceLaw::Optional
        || !matches!(
            native.contract().shape(),
            worth_foundational::facade::AspectShape::Struct(_)
        )
    {
        return Err(WorthQueryPortableOperationContractSpineDenialKind::MissingNativeAspect);
    }
    Ok(WorthQueryPortableOperationTouchScope::RemoveAspect {
        schema: schema.into(),
        entity: entity.into(),
        contract: native.contract().clone(),
    })
}
