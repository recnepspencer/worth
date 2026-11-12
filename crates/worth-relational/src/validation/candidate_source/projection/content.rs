//! Content follows the actual field loan, never a caller-supplied identity.
use super::*;
use crate::content::{NativeContentSourceView, RelationalContentFrame};
impl ManagedIndexSourceProjection<'_> {
    pub(crate) fn with_field_content_at<T>(
        &self,
        row: EntityId,
        kind: crate::identity::data::KindId,
        field: &super::super::ManagedIndexSourceField,
        project: impl FnOnce(Observation<'_>, Option<RelationalContentFrame>) -> T,
    ) -> Result<T, super::super::ManagedIndexSourcePreparationDenial> {
        self.with_field_at(row, kind, field, |value| {
            let content = match &self.backing {
                SourceProjectionBacking::Invariant {
                    relations,
                    interpretation,
                    before,
                    ..
                } => interpretation.content_view(*before)?.read_source(
                    &relations.read_budget(),
                    row,
                    kind,
                    &field.locator,
                    value.value(),
                )?,
                SourceProjectionBacking::Selected {
                    loan, execution, ..
                } => {
                    let root = loan
                        .projection
                        .selected_root()
                        .ok_or(Definition::RecordUnavailable)?;
                    NativeContentSourceView::new(
                        root.retained_content().clone(),
                        root.retained_schema_authority().as_ref().clone(),
                    )
                    .read_selected(
                        execution,
                        row,
                        kind,
                        &field.locator,
                        value.value(),
                    )?
                }
            };
            Ok(project(value, content))
        })
        .map_err(super::super::ManagedIndexSourcePreparationDenial::from)?
    }
}
