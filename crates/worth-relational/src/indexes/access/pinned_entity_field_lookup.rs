//! Field selection captures definitions and generations only from pinned custody.
use super::{prepared_entity_field_lookup::denied, PreparedEntityFieldLookup};
use crate::identity::data::KindId;
use crate::indexes::data::{
    BoundedEntityFieldLookupDenial, BoundedEntityFieldLookupDenialKind as Denial, DerivedIndexId,
};
use crate::runtime::RelationalPinnedRead;
use worth_foundational::facade::AspectFieldLocator;
impl RelationalPinnedRead {
    /// Prepare an installed field lookup from this capability's selected handles.
    /// A cold index stays cold even if its owner later builds a generation.
    pub fn prepare_entity_field_lookup(
        &self,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
        crate::execution::check_request(execution)
            .map_err(|cause| denied(Denial::Execution(cause), index))?;
        let selected = self
            .indexes
            .get(&index)
            .ok_or_else(|| denied(Denial::IndexNotInstalled, index))?;
        PreparedEntityFieldLookup::pin(
            self.context.runtime_instance_id,
            self.basis.clone(),
            selected.definition.clone(),
            kind,
            locator,
            self.instrumentation.clone(),
            |_| selected.generation.clone(),
        )
    }
}
