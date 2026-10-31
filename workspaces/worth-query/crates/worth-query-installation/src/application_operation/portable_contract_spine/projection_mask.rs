//! Interpret declared operation reads through their actual portable Native binding.
use super::{SpineResult, WorthQueryPortableOperationContractSpineDenialKind as Denial};
use crate::package::WorthQueryPortableNativeAspectContractRecord;
use worth_foundational::facade::{
    aspects, AspectBinding, AspectMask, AspectShape, FieldKey, ProjectionMask,
};

pub(super) fn compile(
    native: &WorthQueryPortableNativeAspectContractRecord,
    fields: &[FieldKey],
) -> SpineResult<AspectMask<ProjectionMask>> {
    let mask = if matches!(native.contract().shape(), AspectShape::Scalar(_)) {
        let AspectBinding::EntityField { field } = native.binding() else {
            return Err(Denial::InvalidProjectionMask);
        };
        if fields != std::slice::from_ref(field) {
            return Err(Denial::InvalidProjectionMask);
        }
        aspects().projection_mask().whole_aspect()
    } else {
        aspects()
            .projection_mask()
            .fields(fields.iter().map(FieldKey::as_str))
            .map_err(|_| Denial::InvalidProjectionMask)?
    };
    native
        .contract()
        .admits_projection_mask(&mask)
        .map_err(|_| Denial::InvalidProjectionMask)?;
    Ok(mask)
}
