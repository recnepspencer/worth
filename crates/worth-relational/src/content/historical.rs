//! Historical bytes are selected by the exact authoritative field patch.
use super::{
    control, field_references, NativeContentSourceView, RelationalContentDenial as Denial,
    RelationalContentFrame,
};
use crate::{
    history::data::CanonicalCommitEnvelope, identity::data::EntityId, transactions::data::RecordRef,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl CanonicalCommitEnvelope {
    /// Data custody only. A decoded envelope without complete admission refuses.
    pub fn content_for_field(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        row: EntityId,
        entity: &str,
        locator: &AspectFieldLocator,
    ) -> Result<RelationalContentFrame, Denial> {
        control::run(execution, |control| {
            let (directory, schema) = self.content.historical_parts()?;
            let [key] = locator.field_path().fields() else {
                return Err(Denial::FormatBindingMissing);
            };
            let record = RecordRef::Entity(row);
            let mut selected_kind = None;
            for kind in schema.registry().entity_kinds.values() {
                control.checkpoint(
                    kind.kind_name
                        .len()
                        .checked_add(entity.len())
                        .and_then(|n| n.checked_add(1))
                        .ok_or(Denial::ArithmeticOverflow)? as u64,
                )?;
                if kind.kind_name == entity && selected_kind.replace(kind.kind_id).is_some() {
                    return Err(Denial::EncodingInconsistency);
                }
            }
            let kind = selected_kind.ok_or(Denial::FormatBindingMissing)?;
            let declared = field_references::resolve_aspect(
                control,
                schema.registry(),
                &record,
                kind,
                locator.aspect().aspect_key(),
            )?;
            let mut selected = None;
            for patch in &self.patch.authoritative_record_patches {
                control.checkpoint(1)?;
                if patch.target != record {
                    continue;
                }
                if selected.replace(patch).is_some() {
                    return Err(Denial::EncodingInconsistency);
                }
            }
            let patch = selected.ok_or(Denial::MissingContent)?;
            let (effect, location) = patch.authoritative_patch.field_effect_with_value_location(
                &declared.contract,
                &declared.binding,
                key,
                |work| control.checkpoint(work),
            )?;
            if effect != crate::publication::patch::data::PublishedAuthoritativeFieldEffect::Present
            {
                return Err(Denial::MissingContent);
            }
            let value = patch
                .authoritative_patch
                .field_value_at(location.ok_or(Denial::EncodingInconsistency)?)
                .ok_or(Denial::EncodingInconsistency)?;
            let (RecordRef::Entity(row), AspectValue::ContentRef(_)) = (&record, value) else {
                return Err(Denial::FormatMismatch);
            };
            NativeContentSourceView::new(directory.clone(), schema.clone())
                .read(control, *row, kind, locator, Some(value))?
                .ok_or(Denial::MissingContent)
        })
    }
}
