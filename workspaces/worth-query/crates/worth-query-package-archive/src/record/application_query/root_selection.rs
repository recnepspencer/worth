use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathGuard, ApplicationQueryRootPathMeaning, ApplicationQueryRootPathStep,
    RelationDirection, WorthQueryPortableApplicationQueryRootPathGuardParts,
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
    output.text(step.relation())?;
    output.text(step.from())?;
    output.text(step.to())?;
    output.u16(match step.direction() {
        RelationDirection::Forward => 1,
        RelationDirection::Reverse => 2,
    })
}
fn decode_step(input: &mut BinaryInput<'_>) -> Result<ApplicationQueryRootPathStep, Denial> {
    let relation = input.text()?.to_owned();
    let from = input.text()?.to_owned();
    let to = input.text()?.to_owned();
    let direction = match input.u16()? {
        1 => RelationDirection::Forward,
        2 => RelationDirection::Reverse,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(ApplicationQueryRootPathStep::from_untrusted_fields(
        relation, from, to, direction,
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
