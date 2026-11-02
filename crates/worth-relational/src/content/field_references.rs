//! Interpret actual fields using the selected schema's canonical declarations.
use super::{control, RelationalContentDenial as Denial, RelationalContentFormat as Format};
use crate::validation::engine::state_view::InvariantStateView;
use crate::{schema::data::RelationalSchemaRegistry, transactions::data::RecordRef};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectShape, AspectValue, ContentRefId, ContractValidatedAspectValueView, ScalarAspectType,
};

pub(super) fn visit(
    execution: &ExecutionRequest<'_, '_>,
    schema: &RelationalSchemaRegistry,
    view: &InvariantStateView<'_>,
    record: &RecordRef,
    callback: &mut dyn FnMut(ContentRefId, &Format) -> Result<(), Denial>,
) -> Result<(), Denial> {
    control::run(execution, |control| {
        control.checkpoint(1)?;
        let (aspects, kind) = match record {
            RecordRef::Entity(id) => (
                view.entity_aspect_state(*id),
                view.entity_metadata(*id).map(|meta| meta.kind_id),
            ),
            RecordRef::Relation(id) => (
                view.relation_aspect_state(*id),
                view.relation_metadata(*id).map(|meta| meta.kind_id),
            ),
        };
        let Some(aspects) = aspects else {
            return Ok(());
        };
        for (aspect_key, value) in aspects.aspects().entries() {
            control.checkpoint(1)?;
            match value.view() {
                ContractValidatedAspectValueView::Scalar(AspectValue::ContentRef(_)) => {
                    return Err(Denial::FormatBindingMissing)
                }
                ContractValidatedAspectValueView::Scalar(_) => {}
                ContractValidatedAspectValueView::Struct(values) => {
                    for (key, value) in values.fields() {
                        control.checkpoint(1)?;
                        let AspectValue::ContentRef(identity) = value else {
                            continue;
                        };
                        let kind = kind.ok_or(Denial::EncodingInconsistency)?;
                        control.checkpoint(
                            (usize::BITS
                                - (schema.entity_kinds.len() + schema.relation_kinds.len())
                                    .leading_zeros()
                                + 1) as u64
                                * 8,
                        )?;
                        let declarations = match record {
                            RecordRef::Entity(_) => {
                                &schema
                                    .entity_kinds
                                    .get(&kind)
                                    .ok_or(Denial::FormatBindingMissing)?
                                    .aspect_contract_declarations
                            }
                            RecordRef::Relation(_) => {
                                &schema
                                    .relation_kinds
                                    .get(&kind)
                                    .ok_or(Denial::FormatBindingMissing)?
                                    .aspect_contract_declarations
                            }
                        };
                        let height = usize::BITS - declarations.aspects.len().leading_zeros() + 1;
                        control.checkpoint(u64::from(height) * aspect_key.as_str().len() as u64)?;
                        let index = declarations
                            .aspects
                            .binary_search_by(|binding| binding.contract.key().cmp(aspect_key))
                            .map_err(|_| Denial::FormatBindingMissing)?;
                        let AspectShape::Struct(shape) =
                            declarations.aspects[index].contract.shape()
                        else {
                            return Err(Denial::FormatBindingMissing);
                        };
                        let height = usize::BITS - shape.fields().len().leading_zeros() + 1;
                        control.checkpoint(u64::from(height) * key.as_str().len() as u64)?;
                        let index = shape
                            .fields()
                            .binary_search_by(|field| field.key().cmp(key))
                            .map_err(|_| Denial::FormatBindingMissing)?;
                        let field = &shape.fields()[index];
                        if field.value_type() != ScalarAspectType::ContentRef {
                            return Err(Denial::FormatBindingMissing);
                        }
                        let binding = field.codec().ok_or(Denial::FormatBindingMissing)?;
                        control.checkpoint(schema.content_formats.lookup_bound())?;
                        callback(*identity, schema.content_formats.resolve(binding)?)?;
                    }
                }
            }
        }
        Ok(())
    })
}
