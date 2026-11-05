//! Concrete content checks retain the exact original first refusal.
use {crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control, crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop};
use {crate::domain_computation::primary_graph::application_query::projection::WorthQueryApplicationProjectedField, crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenial};
use worth_relational::facade::content::{
    RelationalContentDenial as Native, RelationalContentFrame,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexContentReadDenial {
    Projection(WorthQueryApplicationProjectionDenial),
    Controlled(Stop),
}
impl From<Stop> for WorthQueryManagedIndexContentReadDenial {
    fn from(cause: Stop) -> Self {
        Self::Controlled(cause)
    }
}
impl From<WorthQueryApplicationProjectionDenial> for WorthQueryManagedIndexContentReadDenial {
    fn from(cause: WorthQueryApplicationProjectionDenial) -> Self {
        Self::Projection(cause)
    }
}
impl std::fmt::Display for WorthQueryManagedIndexContentReadDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "managed Source content read denied: {self:?}")
    }
}
impl std::error::Error for WorthQueryManagedIndexContentReadDenial {}

pub(super) fn admitted<'a>(
    projected: &'a WorthQueryApplicationProjectedField,
    binding: &worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &Control<'_>,
) -> Result<&'a RelationalContentFrame, Stop> {
    let refuse = |cause| control.refuse(Stop::Content(cause));
    let frame = projected
        .content()
        .ok_or_else(|| refuse(Native::MissingContent))?;
    if projected.value() != &worth_foundational::facade::AspectValue::ContentRef(frame.identity()) {
        return Err(refuse(Native::EncodingInconsistency));
    }
    let expected = binding
        .content_format()
        .ok_or_else(|| refuse(Native::FormatBindingMissing))?
        .declaration();
    let maximum = usize::try_from(expected.maximum_frame_bytes)
        .map_err(|_| refuse(Native::ArithmeticOverflow))?;
    match control.source_budget() {
        Some(source) => frame
            .require_source_contract(source, &expected.name, expected.version, maximum)
            .map_err(|cause| {
                use worth_relational::facade::runtime::{
                    DerivedRelationReadDenial as Source,
                    ManagedIndexSourcePreparationDenial as Read,
                };
                control.refuse(match cause {
                    Read::Content(cause) => Stop::Content(cause),
                    Read::Source(Source::Request(cause)) => Stop::MaterialStorage(cause),
                    Read::Source(Source::Structural(cause)) => Stop::NativeSource(cause),
                    _ => Stop::Content(Native::EncodingInconsistency),
                })
            })?,
        None => frame
            .require_contract(execution, &expected.name, expected.version, maximum)
            .map_err(refuse)?,
    }
    Ok(frame)
}

pub(super) fn admit_field(control: &Control<'_>, parts: &[&str]) -> Result<(), Stop> {
    let work = parts
        .iter()
        .try_fold(
            (size_of::<usize>() + size_of::<crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot>()) as u64,
            |sum, part| sum.checked_add(part.len() as u64),
        )
        .and_then(|work| work.checked_mul(2))
        .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
    control.checkpoint(work)
}
