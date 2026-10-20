//! View input admission pins before lending the prepared field lookup.
use super::super::prepared_entity_field_lookup::denied;
use super::{IndexAccess, PreparedEntityFieldLookup};
use crate::identity::data::KindId;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Denial, BoundedEntityFieldLookupOutcome,
    BoundedIndexParityMode, DerivedIndexId,
};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl IndexAccess<'_> {
    pub fn prepare_entity_field_lookup(
        &self,
        view: &VisibilityProjectionView,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
        crate::execution::check_request(execution)
            .map_err(|cause| denied(Denial::Execution(cause), index))?;
        if !view.is_exact_basis() || !view.is_from_runtime(self.runtime) {
            return Err(denied(Denial::SnapshotUnavailable, index));
        }
        let pinned = self.runtime.read_truth().pin_view_read_capability_checked(
            view,
            &[index],
            execution,
            |definition| PreparedEntityFieldLookup::validate_definition(definition, locator),
            |stop| {
                use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
                let kind = match stop {
                    Pin::Request(stop) => {
                        Denial::Execution(crate::execution::denial::kernel_stop(stop))
                    }
                    Pin::SnapshotUnavailable => Denial::SnapshotUnavailable,
                    Pin::IndexNotInstalled(_) => Denial::IndexNotInstalled,
                };
                denied(kind, index)
            },
        )?;
        pinned.prepare_entity_field_lookup(index, kind, locator, execution)
    }

    /// Validate the installed owner before lending the already-pinned lookup.
    pub fn execute_prepared_entity_field_lookup<E: worth_execution::ChargedBytes>(
        &self,
        prepared: &PreparedEntityFieldLookup,
        value: &AspectValue,
        limit: usize,
        parity: BoundedIndexParityMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        check_live: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldLookupOutcome, Stop<E>> {
        if !prepared.belongs_to_owner(self.runtime.runtime_instance_id()) {
            return Err(Stop::ExactBasisRequired);
        }
        prepared.execute_prepared_entity_field_lookup(value, limit, parity, execution, check_live)
    }
}
