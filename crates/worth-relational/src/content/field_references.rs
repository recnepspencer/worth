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
                        let format = resolve_field_format(
                            control,
                            schema,
                            record,
                            kind.ok_or(Denial::EncodingInconsistency)?,
                            aspect_key,
                            key,
                        )?;
                        callback(*identity, format)?;
                    }
                }
            }
        }
        Ok(())
    })
}

pub(super) fn resolve_field_format<'schema>(
    control: &mut super::RelationalContentControl<'_, '_, '_>,
    schema: &'schema RelationalSchemaRegistry,
    record: &RecordRef,
    kind: crate::identity::data::KindId,
    aspect_key: &worth_foundational::facade::AspectKey,
    key: &worth_foundational::facade::FieldKey,
) -> Result<&'schema Format, Denial> {
    let declared = resolve_aspect(control, schema, record, kind, aspect_key)?;
    let AspectShape::Struct(shape) = declared.contract.shape() else {
        return Err(Denial::FormatBindingMissing);
    };
    let height = usize::BITS - shape.fields().len().leading_zeros() + 1;
    control.checkpoint(
        u64::from(height)
            .checked_mul(key.as_str().len() as u64)
            .ok_or(Denial::ArithmeticOverflow)?,
    )?;
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
    schema.content_formats.resolve(binding)
}

pub(super) fn resolve_aspect<'schema>(
    control: &mut super::RelationalContentControl<'_, '_, '_>,
    schema: &'schema RelationalSchemaRegistry,
    record: &RecordRef,
    kind: crate::identity::data::KindId,
    aspect_key: &worth_foundational::facade::AspectKey,
) -> Result<&'schema crate::schema::data::DeclaredAspectContractBinding, Denial> {
    control.checkpoint(
        u64::from(
            usize::BITS
                - schema
                    .entity_kinds
                    .len()
                    .checked_add(schema.relation_kinds.len())
                    .ok_or(Denial::ArithmeticOverflow)?
                    .leading_zeros()
                + 1,
        ) * 8,
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
    control.checkpoint(
        u64::from(height)
            .checked_mul(aspect_key.as_str().len() as u64)
            .ok_or(Denial::ArithmeticOverflow)?,
    )?;
    let index = declarations
        .aspects
        .binary_search_by(|binding| binding.contract.key().cmp(aspect_key))
        .map_err(|_| Denial::FormatBindingMissing)?;
    Ok(&declarations.aspects[index])
}
