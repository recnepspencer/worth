use worth_query_declaration::facade::application_query::{
    ApplicationQueryContinuationTarget, ApplicationQueryLiveCauseContract,
    ApplicationQueryLiveResourceContract, ApplicationQueryOrderingDirection,
    ApplicationQueryOrderingTerm, ApplicationQueryParameterDefinition, ApplicationQueryPredicate,
    ErasedApplicationQueryDefinition, WorthQueryPortableApplicationQueryContinuationParts,
    WorthQueryPortableApplicationQueryLiveCauseParts,
    WorthQueryPortableApplicationQueryOrderingParts, WorthQueryPortableApplicationQueryParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::foundational_aspect;
use super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::super::super::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};

mod controls;
mod disclosure;
mod result_shape;
mod root_selection;

pub(super) fn require_nesting_depth(
    definition: &ErasedApplicationQueryDefinition,
    maximum_depth: u32,
) -> Result<(), Denial> {
    result_shape::require_nesting_depth(definition.result_shape(), maximum_depth)
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), Denial> {
    let parts = definition.parts();
    output.text(parts.name())?;
    for identity in [
        &parts.query_type,
        &parts.parameter_type,
        &parts.result_type,
        &parts.scope_type,
    ] {
        write_type_identity(output, identity)?;
    }
    output.text(parts.root_entity())?;
    output.text(parts.scope_entity())?;
    write_sequence(output, parts.parameters(), write_parameter)?;
    result_shape::write(output, parts.result_shape())?;
    write_sequence(output, parts.root_paths(), root_selection::write_path)?;
    controls::write_cardinality(output, parts.cardinality())?;
    write_sequence(output, parts.predicates(), write_predicate)?;
    write_sequence(output, parts.ordering(), write_ordering)?;
    write_optional(output, parts.continuation(), write_continuation)?;
    write_optional(output, parts.live_cause(), write_live_cause)?;
    controls::write_dependency_ceiling(output, parts.dependency_ceiling())?;
    disclosure::write(output, parts.disclosure())?;
    controls::write_authorization(output, parts.authorization())?;
    controls::write_basis_support(output, parts.basis_support())?;
    controls::write_lanes(output, parts.lanes())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationQueryDefinition, Denial> {
    let name = input.text()?.to_owned();
    let query_type = decode_type_identity(input)?;
    let parameter_type = decode_type_identity(input)?;
    let result_type = decode_type_identity(input)?;
    let scope_type = decode_type_identity(input)?;
    let root_entity = input.text()?.to_owned();
    let scope_entity = input.text()?.to_owned();
    let parameters = decode_sequence(input, budget, 10, |input, _| decode_parameter(input))?;
    let result_shape = result_shape::decode(input, budget)?;
    let root_paths = decode_sequence(input, budget, 12, root_selection::decode_path)?;
    let cardinality = controls::decode_cardinality(input)?;
    let predicates = decode_sequence(input, budget, 18, |input, _| decode_predicate(input))?;
    let ordering = decode_sequence(input, budget, 28, |input, _| decode_ordering(input))?;
    let continuation = decode_optional(input, decode_continuation)?;
    let live_cause = decode_optional(input, decode_live_cause)?;
    let dependency_ceiling = controls::decode_dependency_ceiling(input)?;
    let disclosure = disclosure::decode(input, budget)?;
    let authorization = controls::decode_authorization(input)?;
    let basis_support = controls::decode_basis_support(input)?;
    let lanes = controls::decode_lanes(input)?;
    Ok(ErasedApplicationQueryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationQueryParts {
            name,
            query_type,
            parameter_type,
            result_type,
            scope_type,
            root_entity,
            scope_entity,
            parameters,
            result_shape,
            root_paths,
            cardinality,
            predicates,
            ordering,
            continuation,
            live_cause,
            dependency_ceiling,
            disclosure,
            authorization,
            basis_support,
            lanes,
        },
    ))
}

fn write_parameter(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryParameterDefinition,
) -> Result<(), Denial> {
    output.text(value.name())?;
    foundational_aspect::write_scalar_type(output, value.scalar_family())?;
    output.text(value.value_type())?;
    write_optional(output, value.unit(), |output, unit| output.text(unit))?;
    write_optional(output, value.frame(), |output, frame| output.text(frame))
}
fn decode_parameter(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryParameterDefinition, Denial> {
    Ok(ApplicationQueryParameterDefinition::from_untrusted_fields(
        input.text()?.to_owned(),
        foundational_aspect::decode_scalar_type(input)?,
        decode_type_identity(input)?,
        decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
        decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
    ))
}

fn write_predicate(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryPredicate,
) -> Result<(), Denial> {
    let (entity, aspect, field) = value.field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    output.text(value.parameter())?;
    foundational_aspect::write_scalar_type(output, value.scalar_family())
}
fn decode_predicate(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryPredicate, Denial> {
    Ok(ApplicationQueryPredicate::from_untrusted_fields(
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        input.text()?.to_owned(),
        foundational_aspect::decode_scalar_type(input)?,
    ))
}

fn write_ordering(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryOrderingTerm,
) -> Result<(), Denial> {
    output.text(value.query_type())?;
    output.text(value.slot_type())?;
    let (entity, aspect, field) = value.field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    output.text(value.output_name())?;
    foundational_aspect::write_scalar_type(output, value.scalar_family())?;
    output.text(value.value_type())?;
    output.u16(match value.direction() {
        ApplicationQueryOrderingDirection::Ascending => 1,
        ApplicationQueryOrderingDirection::Descending => 2,
    })
}
fn decode_ordering(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryOrderingTerm, Denial> {
    let query_type = decode_type_identity(input)?;
    let slot_type = decode_type_identity(input)?;
    let entity = input.text()?.to_owned();
    let aspect = input.text()?.to_owned();
    let field = input.text()?.to_owned();
    let output_name = input.text()?.to_owned();
    let scalar_family = foundational_aspect::decode_scalar_type(input)?;
    let value_type = decode_type_identity(input)?;
    let direction = match input.u16()? {
        1 => ApplicationQueryOrderingDirection::Ascending,
        2 => ApplicationQueryOrderingDirection::Descending,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationQueryOrderingTerm::from_untrusted_parts(
        WorthQueryPortableApplicationQueryOrderingParts {
            query_type,
            slot_type,
            entity,
            aspect,
            field,
            output_name,
            scalar_family,
            value_type,
            direction,
        },
    ))
}

fn write_continuation(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryContinuationTarget,
) -> Result<(), Denial> {
    write_type_identity(output, &value.query_identity())?;
    write_type_identity(output, &value.slot_identity())?;
    output.text(value.relation())?;
    output.text(value.parent_entity())?;
    output.text(value.child_entity())?;
    result_shape::write_traversal_direction(output, value.direction())
}
fn decode_continuation(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryContinuationTarget, Denial> {
    Ok(ApplicationQueryContinuationTarget::from_untrusted_parts(
        WorthQueryPortableApplicationQueryContinuationParts {
            query_type: decode_type_identity(input)?,
            slot_type: decode_type_identity(input)?,
            relation: input.text()?.to_owned(),
            parent_entity: input.text()?.to_owned(),
            child_entity: input.text()?.to_owned(),
            direction: result_shape::decode_traversal_direction(input)?,
        },
    ))
}

fn write_live_cause(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryLiveCauseContract,
) -> Result<(), Denial> {
    for text in [
        value.binding_type(),
        value.effect(),
        value.payload_type(),
        value.scope_slot_type(),
    ] {
        output.text(text)?;
    }
    let (entity, aspect, field) = value.scope_field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    output.text(value.scope_value_type())?;
    output.text(value.target_slot_type())?;
    let (entity, aspect, field) = value.target_field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    output.text(value.target_value_type())?;
    let resources = value.resources();
    output.u64(resources.maximum_buffered_causes())?;
    output.u64(resources.maximum_work_per_delivery())?;
    output.u64(resources.maximum_retained_payload_bytes())
}
fn decode_live_cause(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryLiveCauseContract, Denial> {
    Ok(ApplicationQueryLiveCauseContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryLiveCauseParts {
            binding_type: decode_type_identity(input)?,
            effect: input.text()?.to_owned(),
            payload_type: decode_type_identity(input)?,
            scope_slot_type: decode_type_identity(input)?,
            scope_entity: input.text()?.to_owned(),
            scope_aspect: input.text()?.to_owned(),
            scope_field: input.text()?.to_owned(),
            scope_value_type: decode_type_identity(input)?,
            target_slot_type: decode_type_identity(input)?,
            target_entity: input.text()?.to_owned(),
            target_aspect: input.text()?.to_owned(),
            target_field: input.text()?.to_owned(),
            target_value_type: decode_type_identity(input)?,
            resources: ApplicationQueryLiveResourceContract::bounded(
                input.u64()?,
                input.u64()?,
                input.u64()?,
            ),
        },
    ))
}
