pub(super) mod choice;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathGuard, ApplicationQueryRootPathMeaning,
    ApplicationQueryRootPathSourceKind as SourceKind, ApplicationQueryRootPathStep,
    ApplicationQueryRootPathTraversal as Traversal, RelationDirection,
    WorthQueryPortableApplicationQueryRootPathGuardParts,
    WorthQueryPortableApplicationQueryRootPathParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use crate::record::application_schema::wire_vocabulary::{
    decode_type_identity, decode_usize, write_usize,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_aspect;
use crate::record::foundational_value;
use crate::record::sequence::{decode_sequence, write_sequence};

pub(in crate::record) fn write_path(
    output: &mut dyn BinaryEncodingSink,
    path: &ApplicationQueryRootPathMeaning,
) -> Result<(), Denial> {
    output.text(path.start_entity())?;
    output.text(path.terminal_entity())?;
    write_sequence(output, path.steps(), write_step)?;
    write_sequence(output, path.guards(), write_guard)
}

pub(in crate::record) fn decode_path(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationQueryRootPathMeaning, Denial> {
    let start_entity = input.text()?.to_owned();
    let terminal_entity = input.text()?.to_owned();
    let steps = decode_sequence(input, budget, 14, |input, _| decode_step(input))?;
    let guards = decode_sequence(input, budget, 22, |input, _| decode_guard(input))?;
    Ok(ApplicationQueryRootPathMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity,
            terminal_entity,
            steps,
            guards,
        },
    ))
}

fn write_step(
    output: &mut dyn BinaryEncodingSink,
    step: &ApplicationQueryRootPathStep,
) -> Result<(), Denial> {
    if step.from_member().entity().is_none()
        || step.to_member().entity().is_none()
        || (step.source_kind() == SourceKind::Derived && step.traversal() != Traversal::OneHop)
    {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    output.text(step.relation())?;
    output.text(step.from())?;
    output.text(step.to())?;
    output.u16(
        match (step.source_kind(), step.traversal(), step.direction()) {
            (SourceKind::Stored, Traversal::OneHop, RelationDirection::Forward) => 1,
            (SourceKind::Stored, Traversal::OneHop, RelationDirection::Reverse) => 2,
            (SourceKind::Stored, Traversal::Closure, RelationDirection::Forward) => 3,
            (SourceKind::Stored, Traversal::Closure, RelationDirection::Reverse) => 4,
            (SourceKind::Derived, Traversal::OneHop, RelationDirection::Forward) => 5,
            (SourceKind::Derived, Traversal::OneHop, RelationDirection::Reverse) => 6,
            (SourceKind::Derived, Traversal::Closure, _) => {
                return Err(Denial::new(Kind::InvalidRecordShape))
            }
        },
    )
}
fn decode_step(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryRootPathStep, Denial> {
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    let (source_kind, traversal, direction) = match input.u16()? {
        1 => (
            SourceKind::Stored,
            Traversal::OneHop,
            RelationDirection::Forward,
        ),
        2 => (
            SourceKind::Stored,
            Traversal::OneHop,
            RelationDirection::Reverse,
        ),
        3 => (
            SourceKind::Stored,
            Traversal::Closure,
            RelationDirection::Forward,
        ),
        4 => (
            SourceKind::Stored,
            Traversal::Closure,
            RelationDirection::Reverse,
        ),
        5 => (
            SourceKind::Derived,
            Traversal::OneHop,
            RelationDirection::Forward,
        ),
        6 => (
            SourceKind::Derived,
            Traversal::OneHop,
            RelationDirection::Reverse,
        ),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationQueryRootPathStep::from_untrusted_source_fields(
        relation,
        from.into(),
        to.into(),
        source_kind,
        direction,
        traversal,
    ))
}

fn write_guard(
    output: &mut dyn BinaryEncodingSink,
    guard: &ApplicationQueryRootPathGuard,
) -> Result<(), Denial> {
    write_usize(output, guard.after_step())?;
    output.text(guard.entity())?;
    output.text(guard.aspect())?;
    output.text(guard.field())?;
    foundational_aspect::write_scalar_type(output, guard.scalar_family())?;
    output.text(guard.value_type())?;
    foundational_value::write_aspect_value(output, guard.expected())
}
fn decode_guard(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryRootPathGuard, Denial> {
    Ok(ApplicationQueryRootPathGuard::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathGuardParts {
            after_step: decode_usize(input)?,
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value_type: decode_type_identity(input)?,
            expected: foundational_value::decode_aspect_value(input)?,
        },
    ))
}

#[cfg(test)]
mod tests;
