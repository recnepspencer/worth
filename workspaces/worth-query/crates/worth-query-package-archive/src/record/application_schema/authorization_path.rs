use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationComparison, ApplicationAuthorizationPath,
    ApplicationAuthorizationPathEffect, ApplicationAuthorizationPredicate,
    ApplicationAuthorizationTraversal, ApplicationAuthorizationTraversalDirection,
    WorthQueryPortableApplicationAuthorizationPathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts,
    WorthQueryPortableApplicationAuthorizationTraversalParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::decode_budget::RecordDecodeAttempt;
use super::super::foundational_value;
use super::super::sequence::{decode_sequence, write_sequence};

#[path = "authorization_constraint.rs"]
mod authorization_constraint;
#[cfg(test)]
#[path = "authorization_path_tests.rs"]
mod tests;

/// Paths using only equality predicates keep the legacy tags 1 (allow) and 2
/// (deny) and their exact bytes. A path with another comparison or a
/// principal-field constraint is tagged 3 (allow) or 4 (deny) and appends
/// one comparison per predicate and its constraints; an older reader refuses
/// those tags rather than misreading them.
pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    path: &ApplicationAuthorizationPath,
) -> Result<(), Denial> {
    let parts = path.parts();
    let extended = path.uses_extended_conditions();
    output.u16(effect_tag(parts.effect, extended))?;
    output.text(&parts.principal_entity)?;
    output.text(&parts.scope_entity)?;
    write_sequence(output, &parts.traversals, |output, traversal| {
        write_traversal(output, traversal)
    })?;
    write_sequence(output, &parts.predicates, |output, predicate| {
        write_predicate(output, predicate)
    })?;
    if extended {
        write_sequence(output, &parts.predicates, |output, predicate| {
            output.u16(comparison_tag(predicate.comparison()))
        })?;
        write_sequence(output, &parts.field_constraints, |output, constraint| {
            authorization_constraint::write(output, constraint)
        })?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationAuthorizationPath, Denial> {
    let (effect, extended) = effect_from_tag(input.u16()?)?;
    let principal_entity = input.text()?.to_owned();
    let scope_entity = input.text()?.to_owned();
    let traversals = decode_sequence(input, budget, 16, |input, _| decode_traversal(input))?;
    let mut predicates = decode_sequence(input, budget, 18, |input, _| decode_predicate(input))?;
    let mut field_constraints = Vec::new();
    if extended {
        let comparisons = decode_sequence(input, budget, 2, |input, _| {
            comparison_from_tag(input.u16()?)
        })?;
        if comparisons.len() != predicates.len() {
            return Err(Denial::new(Kind::UnsupportedRecordVariant));
        }
        predicates = predicates
            .into_iter()
            .zip(comparisons)
            .map(|(predicate, comparison)| {
                let mut parts = predicate.parts();
                parts.comparison = comparison;
                ApplicationAuthorizationPredicate::from_untrusted_parts(ARCHIVE_DECODE, parts)
            })
            .collect();
        field_constraints = decode_sequence(input, budget, 16, |input, _| {
            authorization_constraint::decode(input)
        })?;
    }
    let path = ApplicationAuthorizationPath::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect,
            principal_entity,
            scope_entity,
            traversals,
            predicates,
            field_constraints,
        },
    );
    // One meaning, one encoding: an extended tag must carry an extended path.
    if extended != path.uses_extended_conditions() {
        return Err(Denial::new(Kind::UnsupportedRecordVariant));
    }
    Ok(path)
}

pub(super) fn write_traversal(
    output: &mut dyn BinaryEncodingSink,
    traversal: &ApplicationAuthorizationTraversal,
) -> Result<(), Denial> {
    let parts = traversal.parts();
    output.text(&parts.relation)?;
    output.text(&parts.from)?;
    output.text(&parts.to)?;
    write_direction(output, parts.direction)
}

pub(super) fn decode_traversal(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationAuthorizationTraversal, Denial> {
    Ok(ApplicationAuthorizationTraversal::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationTraversalParts {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            direction: decode_direction(input)?,
        },
    ))
}

fn write_predicate(
    output: &mut dyn BinaryEncodingSink,
    predicate: &ApplicationAuthorizationPredicate,
) -> Result<(), Denial> {
    let parts = predicate.parts();
    super::wire_vocabulary::write_usize(output, parts.traversal_ordinal)?;
    output.text(&parts.entity)?;
    output.text(&parts.aspect)?;
    output.text(&parts.field)?;
    foundational_value::write_aspect_value(output, &parts.value)
}

fn decode_predicate(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationAuthorizationPredicate, Denial> {
    Ok(ApplicationAuthorizationPredicate::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationAuthorizationPredicateParts {
            traversal_ordinal: super::wire_vocabulary::decode_usize(input)?,
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            value: foundational_value::decode_aspect_value(input)?,
            comparison: ApplicationAuthorizationComparison::Equal,
        },
    ))
}

const fn effect_tag(value: ApplicationAuthorizationPathEffect, extended: bool) -> u16 {
    match (value, extended) {
        (ApplicationAuthorizationPathEffect::Allow, false) => 1,
        (ApplicationAuthorizationPathEffect::Deny, false) => 2,
        (ApplicationAuthorizationPathEffect::Allow, true) => 3,
        (ApplicationAuthorizationPathEffect::Deny, true) => 4,
    }
}

fn effect_from_tag(tag: u16) -> Result<(ApplicationAuthorizationPathEffect, bool), Denial> {
    match tag {
        1 => Ok((ApplicationAuthorizationPathEffect::Allow, false)),
        2 => Ok((ApplicationAuthorizationPathEffect::Deny, false)),
        3 => Ok((ApplicationAuthorizationPathEffect::Allow, true)),
        4 => Ok((ApplicationAuthorizationPathEffect::Deny, true)),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) const fn comparison_tag(value: ApplicationAuthorizationComparison) -> u16 {
    match value {
        ApplicationAuthorizationComparison::Equal => 1,
        ApplicationAuthorizationComparison::AtMost => 2,
        ApplicationAuthorizationComparison::AtLeast => 3,
        ApplicationAuthorizationComparison::StrictlyLess => 4,
        ApplicationAuthorizationComparison::StrictlyGreater => 5,
    }
}

pub(super) fn comparison_from_tag(tag: u16) -> Result<ApplicationAuthorizationComparison, Denial> {
    match tag {
        1 => Ok(ApplicationAuthorizationComparison::Equal),
        2 => Ok(ApplicationAuthorizationComparison::AtMost),
        3 => Ok(ApplicationAuthorizationComparison::AtLeast),
        4 => Ok(ApplicationAuthorizationComparison::StrictlyLess),
        5 => Ok(ApplicationAuthorizationComparison::StrictlyGreater),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn write_direction(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationAuthorizationTraversalDirection,
) -> Result<(), Denial> {
    output.u16(direction_tag(value))
}

const fn direction_tag(value: ApplicationAuthorizationTraversalDirection) -> u16 {
    match value {
        ApplicationAuthorizationTraversalDirection::Forward => 1,
        ApplicationAuthorizationTraversalDirection::Reverse => 2,
    }
}

pub(super) fn decode_direction(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationAuthorizationTraversalDirection, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationAuthorizationTraversalDirection::Forward),
        2 => Ok(ApplicationAuthorizationTraversalDirection::Reverse),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
