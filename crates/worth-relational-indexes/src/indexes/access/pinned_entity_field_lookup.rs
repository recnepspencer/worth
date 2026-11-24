//! Field selection captures definitions and generations only from pinned custody.
use super::prepared_entity_field_lookup::denied;
use super::selected_index::PinnedIndex;
use crate::branch::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind as Denial,
};
use crate::branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::branch::visibility::snapshot_states::resident_basis::ResidentVisibilitySnapshotBasis;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::indexes::data::definition::DerivedIndexId;
use crate::indexes::access::prepared_entity_field_lookup::PreparedEntityFieldLookup;
use worth_foundational::facade::AspectFieldLocator;
pub fn prepare(
    runtime_identity: u64,
    resident: &ResidentVisibilitySnapshotBasis,
    instrumentation: &RuntimeInstrumentation,
    indexes: &std::collections::BTreeMap<DerivedIndexId, PinnedIndex>,
    index: DerivedIndexId,
    kind: KindId,
    locator: &AspectFieldLocator,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
    crate::contracts::execution::request_custody::check_request(execution)
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
