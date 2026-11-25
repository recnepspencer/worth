//! Optional trailing archive extension for related-record absence/existence
//! predicates. Queries with no related predicates emit zero trailing bytes,
//! preserving exact prior wire bytes.
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::{
    decode_budget::RecordDecodeAttempt,
    foundational_aspect, foundational_value,
    sequence::{decode_sequence, require_canonical_sequence, write_sequence},
};
use worth_query_declaration::facade::application_query::{
    validate_related_predicates, ApplicationQueryPredicateExpected,
    ApplicationQueryRelatedPolarity, ApplicationQueryRelatedPredicateMeaning,
    ApplicationQueryRelatedTargetPredicate, RelationDirection,
    WorthQueryPortableApplicationQueryParts,
    WorthQueryPortableApplicationQueryRelatedPredicateParts,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

use super::where_filter;

const MARKER: &[u8; 4] = b"WQRP";
const VERSION: u16 = 1;

pub(super) fn validate(parts: &WorthQueryPortableApplicationQueryParts) -> Result<(), Denial> {
    validate_related_predicates(parts).map_err(|_| Denial::new(Kind::InvalidRecordShape))
}

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    parts: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    let predicates = parts.related_predicates();
    if predicates.is_empty() {
        return Ok(());
    }
    validate(parts)?;
    require_canonical_sequence(predicates)?;
    output.raw_bytes(MARKER)?;
    output.u16(VERSION)?;
    write_sequence(output, predicates, write_one)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<ApplicationQueryRelatedPredicateMeaning>, Denial> {
    if !input.remaining_starts_with(MARKER) {
        return Ok(Vec::new());
    }
    input.take(MARKER.len())?;
    if input.u16()? != VERSION {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    let predicates = decode_sequence(input, budget, 16, decode_one)?;
    if predicates.is_empty() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    require_canonical_sequence(&predicates)?;
    Ok(predicates)
}

fn write_one(
    output: &mut dyn BinaryEncodingSink,
    predicate: &ApplicationQueryRelatedPredicateMeaning,
) -> Result<(), Denial> {
    require_canonical_sequence(predicate.target_predicates())?;
    output.text(predicate.relation())?;
    output.text(predicate.from())?;
    output.text(predicate.to())?;
    output.u8(match predicate.direction() {
        RelationDirection::Forward => 0,
        RelationDirection::Reverse => 1,
    })?;
    output.u8(match predicate.polarity() {
        ApplicationQueryRelatedPolarity::Absent => 0,
        ApplicationQueryRelatedPolarity::Exists => 1,
    })?;
    write_sequence(
        output,
        predicate.target_predicates(),
        write_target_predicate,
    )?;
    match predicate.target_where_filter() {
        None => output.u8(0),
        Some(filter) => {
            output.u8(1)?;
            where_filter::write_program_body(output, filter)
        }
    }
}

fn decode_one(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationQueryRelatedPredicateMeaning, Denial> {
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    let direction = match input.u8()? {
        0 => RelationDirection::Forward,
        1 => RelationDirection::Reverse,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let polarity = match input.u8()? {
        0 => ApplicationQueryRelatedPolarity::Absent,
        1 => ApplicationQueryRelatedPolarity::Exists,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let target_predicates =
        decode_sequence(input, budget, 12, |input, _| decode_target_predicate(input))?;
    require_canonical_sequence(&target_predicates)?;
    let target_where_filter = match input.u8()? {
        0 => None,
        1 => Some(where_filter::decode_program_body(input, budget)?),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation,
                from,
                to,
                direction,
                polarity,
                target_predicates,
                target_where_filter,
            },
        ),
    )
}

fn write_target_predicate(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationQueryRelatedTargetPredicate,
) -> Result<(), Denial> {
    let (entity, aspect, field) = target.field();
    output.text(entity)?;
    output.text(aspect)?;
    output.text(field)?;
    foundational_aspect::write_scalar_type(output, target.scalar_family())?;
    output.u8(match target.presence() {
        ApplicationFieldPresence::Required => 0,
        ApplicationFieldPresence::Optional => 1,
    })?;
    match target.expected() {
        ApplicationQueryPredicateExpected::Parameter(parameter) => {
            if parameter.is_empty() {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            output.u8(0)?;
            output.text(parameter)
        }
        ApplicationQueryPredicateExpected::Value(expected) => {
            if !ApplicationQueryRelatedTargetPredicate::literal_value_is_valid(
                expected,
                target.scalar_family(),
            ) {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            output.u8(1)?;
            foundational_value::write_aspect_value(output, expected)
        }
    }
}

fn decode_target_predicate(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryRelatedTargetPredicate, Denial> {
    let entity = input.text()?.to_owned();
    let aspect = input.text()?.to_owned();
    let field = input.text()?.to_owned();
    let scalar_family = foundational_aspect::decode_scalar_type(input)?;
    let presence = match input.u8()? {
        0 => ApplicationFieldPresence::Required,
        1 => ApplicationFieldPresence::Optional,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    match input.u8()? {
        0 => {
            let parameter = input.text()?.to_owned();
            if parameter.is_empty() {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            Ok(
                ApplicationQueryRelatedTargetPredicate::from_untrusted_parameter(
                    entity,
                    aspect,
                    field,
                    parameter,
                    scalar_family,
                    presence,
                ),
            )
        }
        1 => {
            let expected = foundational_value::decode_aspect_value(input)?;
            if !ApplicationQueryRelatedTargetPredicate::literal_value_is_valid(
                &expected,
                scalar_family,
            ) {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            Ok(
                ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                    entity,
                    aspect,
                    field,
                    expected,
                    scalar_family,
                    presence,
                ),
            )
        }
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

#[cfg(test)]
mod tests;
