//! Original-request storage for both Native and Store source retirement.
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::runtime::PreparedRelationalSourceClose;

/// Vacant original-request-funded storage, with no serving or retired capability.
#[must_use]
pub struct PreparedRelationalSectionClose {
    pub(super) native: PreparedRelationalSourceClose,
    pub(super) cold: crate::cold_source::StoreColdSourceClose,
}
impl PreparedRelationalSectionClose {
    pub fn prepare(request: &ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        Ok(Self {
            native: PreparedRelationalSourceClose::prepare(request)?,
            cold: crate::cold_source::StoreColdSourceClose::prepare(request)?,
        })
    }
}
