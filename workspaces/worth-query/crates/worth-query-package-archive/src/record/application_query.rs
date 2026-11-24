use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryContinuationTarget, ApplicationQueryLiveCauseContract,
    ApplicationQueryLiveTargetMode, ApplicationQueryOrderingDirection,
    ApplicationQueryOrderingTerm, ApplicationQueryParameterDefinition, ApplicationQueryPredicate,
    ApplicationQueryPredicateExpected, ErasedApplicationQueryDefinition,
    WorthQueryPortableApplicationQueryContinuationParts,
    WorthQueryPortableApplicationQueryLiveCauseParts,
    WorthQueryPortableApplicationQueryOrderingParts, WorthQueryPortableApplicationQueryParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::application_schema::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};
use super::decode_budget::RecordDecodeAttempt;
use super::foundational_aspect;
use super::foundational_value;
use super::sequence::{decode_sequence, write_sequence};

mod components;
pub(in crate::record) mod controls;
pub(in crate::record) mod disclosure;
mod field_presence;
mod live_whole_result;
mod managed_index;
mod orbit;
mod related_predicate;
pub(in crate::record) mod result_shape;
mod root_selection;
mod where_filter;
pub(in crate::record) use root_selection::{
    decode_path as decode_root_path, write_path as write_root_path,
};

pub(super) fn require_nesting_depth(
    definition: &ErasedApplicationQueryDefinition,
    maximum_depth: u32,
) -> Result<(), Denial> {
    root_selection::choice::require_exclusive(definition.parts())?;
    result_shape::require_nesting_depth(definition.result_shape(), maximum_depth)
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationQueryDefinition,
) -> Result<(), Denial> {
    let parts = definition.parts();
    root_selection::choice::require_exclusive(parts)?;
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
    root_selection::choice::write(output, parts)?;
    controls::write_cardinality(output, parts.cardinality())?;
    write_sequence(output, parts.predicates(), write_predicate)?;
    write_sequence(output, parts.ordering(), write_ordering)?;
    write_optional(output, parts.continuation(), write_continuation)?;
    write_optional(output, parts.live_cause(), write_live_cause)?;
    disclosure::write(output, parts.disclosure())?;
    controls::write_authorization(output, parts.authorization())?;
    controls::write_basis_support(output, parts.basis_support())?;
    controls::write_lanes(output, parts.lanes())?;
    where_filter::write(output, parts.where_filter())?;
    related_predicate::write(output, parts.related_predicates())?;
    live_whole_result::write(output, parts)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationQueryDefinition, Denial> {
    decode_with_shape_depth(input, budget, 1)
}

pub(super) fn decode_nested(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationQueryDefinition, Denial> {
    decode_with_shape_depth(input, budget, 2)
}

fn decode_with_shape_depth(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    depth: u32,
) -> Result<ErasedApplicationQueryDefinition, Denial> {
    let name = input.text()?.to_owned();
    let query_type = decode_type_identity(input)?;
    let parameter_type = decode_type_identity(input)?;
    let result_type = decode_type_identity(input)?;
    let scope_type = decode_type_identity(input)?;
    let root_entity = input.text()?.to_owned();
    let scope_entity = input.text()?.to_owned();
    let parameters = decode_sequence(input, budget, 10, |input, _| decode_parameter(input))?;
    let result_shape = result_shape::decode_at_depth(input, budget, depth)?;
    let root_paths = decode_sequence(input, budget, 12, root_selection::decode_path)?;
    let (orbit, component_selection, managed_index) = root_selection::choice::decode(input)?;
    let cardinality = controls::decode_cardinality(input)?;
    let predicates = decode_sequence(input, budget, 18, |input, _| decode_predicate(input))?;
    let ordering = decode_sequence(input, budget, 28, |input, _| decode_ordering(input))?;
    let continuation = decode_optional(input, decode_continuation)?;
    let live_cause = decode_optional(input, decode_live_cause)?;
    let disclosure = disclosure::decode(input, budget)?;
    let authorization = controls::decode_authorization(input)?;
    let basis_support = controls::decode_basis_support(input)?;
    let lanes = controls::decode_lanes(input)?;
    let where_filter = where_filter::decode(input, budget)?;
    if let Some(filter) = where_filter.as_ref() {
        filter
            .validate_against_parameters(&parameters)
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    }
    let related_predicates = related_predicate::decode(input, budget, &root_entity, &parameters)?;
    let live_whole_result = live_whole_result::decode(
        input,
        lanes.live_enabled(),
        live_cause.is_some(),
        orbit.is_some()
            || component_selection.is_some()
            || managed_index.is_some()
            || continuation.is_some(),
    )?;
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
            orbit,
            component_selection,
            managed_index,
            cardinality,
            predicates,
            where_filter,
            related_predicates,
            ordering,
            continuation,
            live_cause,
            live_whole_result,
            disclosure,
            authorization,
            basis_support,
            lanes,
        },
    ))
}

// This prefix occupies a tag outside the scalar vocabulary. Required parameter
// records keep their original byte sequence.
const OPTIONAL_PARAMETER_TAG: u16 = 0;

fn write_parameter(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryParameterDefinition,
) -> Result<(), Denial> {
    output.text(value.name())?;
    if value.is_optional() {
        output.u16(OPTIONAL_PARAMETER_TAG)?;
    }
    foundational_aspect::write_scalar_type(output, value.scalar_family())?;
    output.text(value.value_type())?;
    write_optional(output, value.unit(), |output, unit| output.text(unit))?;
    write_optional(output, value.frame(), |output, frame| output.text(frame))
}
fn decode_parameter(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryParameterDefinition, Denial> {
    let name = input.text()?.to_owned();
    let tag = input.u16()?;
    let optional = tag == OPTIONAL_PARAMETER_TAG;
    let scalar = if optional {
        foundational_aspect::decode_scalar_type(input)?
    } else {
        foundational_aspect::decode_scalar_type_tag(tag)?
    };
    if optional && scalar == ScalarAspectType::Null {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let parameter = ApplicationQueryParameterDefinition::from_untrusted_fields(
        name,
        scalar,
        decode_type_identity(input)?,
        decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
        decode_optional(input, |input| Ok(input.text()?.to_owned()))?,
    );
    Ok(if optional {
        parameter.with_optional_presence()
    } else {
        parameter
    })
}

fn write_predicate(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryPredicate,
) -> Result<(), Denial> {
    let (entity, aspect, field) = value.field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    match value.expected() {
        ApplicationQueryPredicateExpected::Parameter(parameter) => {
            if parameter.is_empty() {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            output.text(parameter)?;
            foundational_aspect::write_scalar_type(output, value.scalar_family())
        }
        ApplicationQueryPredicateExpected::Value(expected) => {
            // An empty parameter name cannot represent a valid parameter;
            // required parameter predicates keep their exact original bytes.
            output.text("")?;
            foundational_aspect::write_scalar_type(output, value.scalar_family())?;
            foundational_value::write_aspect_value(output, expected)
        }
    }
}
fn decode_predicate(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryPredicate, Denial> {
    let entity = input.text()?.to_owned();
    let aspect = input.text()?.to_owned();
    let field = input.text()?.to_owned();
    let parameter = input.text()?.to_owned();
    let scalar_family = foundational_aspect::decode_scalar_type(input)?;
    if parameter.is_empty() {
        let expected = foundational_value::decode_aspect_value(input)?;
        if !ApplicationQueryPredicate::literal_value_is_valid(&expected, scalar_family) {
            return Err(Denial::new(Kind::InvalidRecordShape));
        }
        Ok(ApplicationQueryPredicate::from_untrusted_value(
            entity,
            aspect,
            field,
            expected,
            scalar_family,
        ))
    } else {
        Ok(ApplicationQueryPredicate::from_untrusted_fields(
            entity,
            aspect,
            field,
            parameter,
            scalar_family,
        ))
    }
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
    output.u8(match value.target_mode() {
        ApplicationQueryLiveTargetMode::Root => 0,
        ApplicationQueryLiveTargetMode::Collection => 1,
    })?;
    Ok(())
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
            target_mode: match input.u8()? {
                0 => ApplicationQueryLiveTargetMode::Root,
                1 => ApplicationQueryLiveTargetMode::Collection,
                _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
            },
        },
    ))
}

#[cfg(test)]
mod tests;
