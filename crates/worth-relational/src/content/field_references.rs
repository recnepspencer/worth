//! Interpret actual fields using the selected schema's canonical declarations.
use super::{control, RelationalContentDenial as Denial, RelationalContentFormat as Format};
use crate::identity::data::VersionId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::{record_observation::observe_record, CurrentPartitionAccess};
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectShape, AspectValue, ContentRefId, ContractValidatedAspectValueView, ScalarAspectType,
};

pub(super) fn visit(
    execution: &ExecutionRequest<'_, '_>,
    schema: &RelationalSchemaRegistry,
    state: &dyn CurrentPartitionAccess,
    version: VersionId,
    record: &RecordRef,
    callback: &mut dyn FnMut(ContentRefId, &Format) -> Result<(), Denial>,
) -> Result<(), Denial> {
    control::run(execution, |control| {
        control.checkpoint(1)?;
        let Some(observation) = observe_record(state, record, version) else {
            return Ok(());
        };
        let Some(aspects) = observation.aspects else {
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
                            observation.kind,
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
