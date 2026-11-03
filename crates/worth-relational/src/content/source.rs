//! Field-bound Source views share the actual admitted directory and schema custody.
use super::{
    control, field_references, ContentDirectory, RelationalContentDenial as Denial,
    RelationalContentFrame,
};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::RecordRef;
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Read;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

#[derive(Clone, Debug)]
pub(crate) struct NativeContentSourceView {
    directory: ContentDirectory,
    schema: RelationalBranchRootSchemaAuthority,
}
impl NativeContentSourceView {
    pub(crate) fn new(
        directory: ContentDirectory,
        schema: RelationalBranchRootSchemaAuthority,
    ) -> Self {
        Self { directory, schema }
    }
    pub(crate) fn read(
        &self,
        control: &mut super::RelationalContentControl<'_, '_, '_>,
        row: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: Option<&AspectValue>,
    ) -> Result<Option<RelationalContentFrame>, Denial> {
        let Some(AspectValue::ContentRef(identity)) = value else {
            return Ok(None);
        };
        let [key] = locator.field_path().fields() else {
            return Err(Denial::FormatBindingMissing);
        };
        let format = field_references::resolve_field_format(
            control,
            self.schema.registry(),
            &RecordRef::Entity(row),
            kind,
            locator.aspect().aspect_key(),
            key,
        )?;
        let frame = self
            .directory
            .find_controlled(control, *identity)?
            .ok_or(Denial::MissingContent)?;
        frame.require_contract_controlled(
            control,
            format.descriptor().name(),
            format.descriptor().version(),
            format.maximum_frame_bytes(),
        )?;
        Ok(Some(frame))
    }
    pub(crate) fn read_source(
        &self,
        budget: &crate::validation::data::CustomInvariantReadBudget<'_>,
        row: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: Option<&AspectValue>,
    ) -> Result<Option<RelationalContentFrame>, Read> {
        control::run_source(budget, |control| {
            self.read(control, row, kind, locator, value)
        })
    }
    pub(crate) fn read_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        row: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        value: Option<&AspectValue>,
    ) -> Result<Option<RelationalContentFrame>, Read> {
        control::run(execution, |control| {
            self.read(control, row, kind, locator, value)
        })
        .map_err(|cause| match cause {
            Denial::Execution(cause) => Read::Source(
                crate::derived_relation::DerivedRelationReadDenial::Request(cause),
            ),
            cause => Read::Content(cause),
        })
    }
}
