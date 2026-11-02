//! Ordinary projection retains the same selected Native field's content custody.
use super::{
    control, NativeContentSourceView, RelationalContentDenial as Denial, RelationalContentFrame,
};
use crate::identity::data::{EntityId, KindId};
use crate::runtime::VisibilityProjectionView;
use worth_foundational::facade::{AspectFieldLocator, ContractValidatedAspectValueView};
impl VisibilityProjectionView {
    pub fn retain_entity_field_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        row: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<RelationalContentFrame>, Denial> {
        control::run(execution, |control| {
            let root = self.selected_root().ok_or(Denial::MissingContent)?;
            control.checkpoint(
                self.exact_entity_state_read_work_bound()
                    .ok_or(Denial::MissingContent)?,
            )?;
            self.read_exact_entity_state(execution, row, |metadata, state| {
                if metadata.entity_id != row || metadata.kind_id != kind {
                    return Err(Denial::EncodingInconsistency);
                }
                let [key] = locator.field_path().fields() else {
                    return Err(Denial::FormatBindingMissing);
                };
                let Some(state) = state else {
                    return Ok(None);
                };
                control.checkpoint(
                    (u64::from(usize::BITS - state.aspects().len().leading_zeros()) + 1)
                        .checked_mul(locator.aspect().aspect_key().as_str().len() as u64)
                        .ok_or(Denial::ArithmeticOverflow)?,
                )?;
                let Some(aspect) = state.get(locator.aspect().aspect_key()) else {
                    return Ok(None);
                };
                let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
                    return Err(Denial::FormatBindingMissing);
                };
                control.checkpoint(
                    (u64::from(usize::BITS - fields.len().leading_zeros()) + 1)
                        .checked_mul(key.as_str().len() as u64)
                        .ok_or(Denial::ArithmeticOverflow)?,
                )?;
                NativeContentSourceView::new(
                    root.retained_content().clone(),
                    root.retained_schema_authority().as_ref().clone(),
                )
                .read(control, row, kind, locator, fields.get(key))
            })
            .map_err(|cause| match cause {
                crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                    Denial::Execution(cause)
                }
                crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                    Denial::MissingContent
                }
            })?
            .ok_or(Denial::MissingContent)?
        })
    }
}
