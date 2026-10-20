use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
    ApplicationAuthorizationTraversalDirection,
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

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    path: &ApplicationAuthorizationPath,
) -> Result<(), Denial> {
    let parts = path.parts();
    output.u16(effect_tag(parts.effect))?;
    output.text(&parts.principal_entity)?;
    output.text(&parts.scope_entity)?;
    write_sequence(output, &parts.traversals, |output, traversal| {
        write_traversal(output, traversal)
    })?;
    write_sequence(output, &parts.predicates, |output, predicate| {
        write_predicate(output, predicate)
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationAuthorizationPath, Denial> {
    let effect = effect_from_tag(input.u16()?)?;
    let principal_entity = input.text()?.to_owned();
    let scope_entity = input.text()?.to_owned();
    let traversals = decode_sequence(input, budget, 16, |input, _| decode_traversal(input))?;
    let predicates = decode_sequence(input, budget, 18, |input, _| decode_predicate(input))?;
    Ok(ApplicationAuthorizationPath::from_untrusted_parts(
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect,
            principal_entity,
            scope_entity,
            traversals,
            predicates,
        },
    ))
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
        WorthQueryPortableApplicationAuthorizationPredicateParts {
            traversal_ordinal: super::wire_vocabulary::decode_usize(input)?,
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            value: foundational_value::decode_aspect_value(input)?,
        },
    ))
}

const fn effect_tag(value: ApplicationAuthorizationPathEffect) -> u16 {
    match value {
        ApplicationAuthorizationPathEffect::Allow => 1,
        ApplicationAuthorizationPathEffect::Deny => 2,
    }
}

fn effect_from_tag(tag: u16) -> Result<ApplicationAuthorizationPathEffect, Denial> {
    match tag {
        1 => Ok(ApplicationAuthorizationPathEffect::Allow),
        2 => Ok(ApplicationAuthorizationPathEffect::Deny),
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
