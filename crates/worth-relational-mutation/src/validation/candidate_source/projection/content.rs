//! Content follows the actual field loan, never a caller-supplied identity.
use super::*;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
use worth_relational_branch::internal_port::content::source::NativeContentSourceView;
use worth_relational_storage::internal_port::content::frame::RelationalContentFrame;
impl ManagedIndexSourceProjection<'_> {
    pub(crate) fn with_field_content_at<T>(
        &self,
        row: EntityId,
        kind: worth_relational_contracts::internal_port::identity::data::primitives::KindId,
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
                } => relations.read_budget().read_field_content(
                    interpretation.content_view(*before)?,
                    row,
                    kind,
                    &field.locator,
                    value.value(),
                )?,
                SourceProjectionBacking::Selected { .. }
                | SourceProjectionBacking::Applied { .. } => {
                    let (projection, execution) = self
                        .physical_projection()
                        .ok_or(Definition::ForeignObservation)?;
                    let root = projection
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
