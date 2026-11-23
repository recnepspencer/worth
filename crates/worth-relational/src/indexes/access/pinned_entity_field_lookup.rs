//! Field selection captures definitions and generations only from pinned custody.
use super::prepared_entity_field_lookup::denied;
use super::selected_index::PinnedIndex;
use crate::identity::data::KindId;
use crate::indexes::access::prepared_entity_field_lookup::PreparedEntityFieldLookup;
use crate::indexes::data::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind as Denial, DerivedIndexId,
};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::visibility::snapshot_states::ResidentVisibilitySnapshotBasis;
use worth_foundational::facade::AspectFieldLocator;
pub(crate) fn prepare(
    runtime_identity: u64,
    resident: &ResidentVisibilitySnapshotBasis,
    instrumentation: &RuntimeInstrumentation,
    indexes: &std::collections::BTreeMap<DerivedIndexId, PinnedIndex>,
    index: DerivedIndexId,
    kind: KindId,
    locator: &AspectFieldLocator,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
    crate::execution::request_custody::check_request(execution)
        .map_err(|cause| denied(Denial::Execution(cause), index))?;
    let selected = indexes
        .get(&index)
        .ok_or_else(|| denied(Denial::IndexNotInstalled, index))?;
    PreparedEntityFieldLookup::pin(
        runtime_identity,
        resident.clone(),
        selected.definition.clone(),
        kind,
        locator,
        instrumentation.clone(),
        |_| selected.generation.clone(),
    )
}
