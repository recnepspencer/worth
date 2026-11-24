//! Add authoritative endpoint operations to the original mutation candidate.
use super::field_authoring_candidate::{self as allocation, FieldCandidate};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractPlan;
use crate::contracts::schema::portable_patch::endpoints::append;

pub(super) fn append_authoritative_endpoints(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    mut candidate: FieldCandidate,
    plan: Option<&LoweredAspectContractPlan>,
    source: EntityId,
    target: EntityId,
) -> Result<FieldCandidate, super::record_aspect_patch::MutationPatchDenial> {
    append(
        &mut candidate.operations,
        &mut candidate.funding,
        plan,
        source,
        target,
        |memory, bytes| allocation::resize(request, memory, bytes),
    )?;
    Ok(candidate)
}
