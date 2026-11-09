//! Fresh admission requires the same Native birth and an actually empty Home cut.
use super::*;
use crate::durability::{
    WorldDurableMediumBinding, WorldDurableMediumDenial, WorldReadmissionDenial,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::FreshRelationalSourceAdmission, runtime::RelationalRuntime,
};

fn request_denial(cause: worth_execution::WorkCeilingDenial) -> WorldReadmissionDenial {
    WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
}

pub(super) fn admit_source(
    medium: Arc<dyn crate::durability::RuntimeWorldDurableMedium>,
    source: Box<dyn crate::durability::WorldCheckpointReadSource>,
    request: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    admission: &FreshRelationalSourceAdmission,
) -> Result<Arc<WorldDurableMediumBinding>, WorldReadmissionDenial> {
    request.consult().map_err(request_denial)?;
    admission
        .validate_fresh_source_runtime(request, native)
        .map_err(WorldReadmissionDenial::NativeFreshSource)?;
    source
        .read(request)
        .map_err(WorldReadmissionDenial::Medium)?
        .require_fresh_empty()?;
    // No owner is constructed, and no recovered effects are discovered,
    // until the original request and both actual source admissions agree.
    request.consult().map_err(request_denial)?;
    WorldDurableMediumBinding::new(medium, request).map_err(WorldReadmissionDenial::Medium)
}
