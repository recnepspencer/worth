//! Capture exact field content from its immutable original basis.
use super::{
    control, NativeContentSourceView, RelationalContentDenial as Denial, RelationalContentFrame,
};
use crate::identity::data::{EntityId, KindId};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl crate::branch::AdmittedRelationalBranchBasis {
    /// Capture the content of an actual field from this immutable original basis.
    pub fn retain_field_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        row: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        expected: worth_foundational::facade::ContentRefId,
    ) -> Result<RelationalContentFrame, Denial> {
        use crate::storage::overlay::PartitionAccess;
        let selected = crate::branch::SelectedRelationalBranchState::from_admitted_basis(self);
        selected
            .demand_full_partition(execution, row.partition_id)
            .map_err(|cause| match cause {
                crate::storage::residency::RelationalResidencyDemandFailure::Request(cause) => {
                    Denial::Execution(cause)
                }
                crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(
                    cause,
                ) => Denial::Execution(
                    crate::execution::RelationalExecutionDenialCause::StorageUnavailable(cause),
                ),
            })?;
        let view = crate::validation::engine::state_view::InvariantStateView::new(
            &selected,
            self.observation().version_id(),
        );
        control::run(execution, |control| {
            control.checkpoint(128)?;
            let metadata = view
                .entity_metadata(row)
                .ok_or(Denial::EncodingInconsistency)?;
            if metadata.kind_id != kind {
                return Err(Denial::EncodingInconsistency);
            }
            let [key] = locator.field_path().fields() else {
                return Err(Denial::FormatBindingMissing);
            };
            let state = view
                .entity_aspect_state(row)
                .ok_or(Denial::EncodingInconsistency)?;
            let aspect_work = (u64::from(usize::BITS - state.aspects().len().leading_zeros()) + 1)
                .checked_mul(locator.aspect().aspect_key().as_str().len() as u64)
                .ok_or(Denial::ArithmeticOverflow)?;
            control.checkpoint(aspect_work)?;
            let aspect = state
                .get(locator.aspect().aspect_key())
                .ok_or(Denial::EncodingInconsistency)?;
            let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) =
                aspect.view()
            else {
                return Err(Denial::FormatBindingMissing);
            };
            control.checkpoint(
                (u64::from(usize::BITS - fields.len().leading_zeros()) + 2)
                    .checked_mul(key.as_str().len() as u64)
                    .ok_or(Denial::ArithmeticOverflow)?,
            )?;
            let value = fields.get(key).ok_or(Denial::EncodingInconsistency)?;
            if value != &AspectValue::ContentRef(expected) {
                return Err(Denial::EncodingInconsistency);
            }
            NativeContentSourceView::new(
                self.inner.root.retained_content().clone(),
                self.inner.root.retained_schema_authority().as_ref().clone(),
            )
            .read(control, row, kind, locator, Some(value))?
            .ok_or(Denial::MissingContent)
        })
    }
}
