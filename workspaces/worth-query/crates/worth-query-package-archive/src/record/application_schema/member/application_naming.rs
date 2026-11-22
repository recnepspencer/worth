use super::*;
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::facade::application_naming::{
    ApplicationNameRoleDependency, ApplicationNameSubjectKind, ErasedApplicationNameScheme,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationNameScheme,
) -> Result<(), Denial> {
    output.u16(1)?;
    output.text(definition.scheme_id())?;
    output.u8(match definition.subject_kind() {
        ApplicationNameSubjectKind::Entity => 1,
        ApplicationNameSubjectKind::Relation => 2,
        ApplicationNameSubjectKind::Group => 3,
    })?;
    output.text(definition.subject_name())?;
    write_sequence(output, definition.role_order(), |output, role| {
        output.text(role)
    })?;
    write_sequence(output, definition.role_dependencies(), |output, dep| {
        output.text(dep.role())?;
        output.text(dep.depends_on())
    })?;
    output.u8(u8::from(definition.split_capable()))?;
    match definition.split_discriminator_identity() {
        Some(identity) => {
            output.u8(1)?;
            output.text(identity)?;
        }
        None => output.u8(0)?,
    }
    write_sequence(
        output,
        definition.discriminator_referenced_roles(),
        |output, role| output.text(role),
    )?;
    output.u8(u8::from(definition.require_authored_occurrence()))
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    budget.require_nesting_depth(2)?;
    let scheme_id = input.text()?.to_owned();
    let subject_kind = match input.u8()? {
        1 => ApplicationNameSubjectKind::Entity,
        2 => ApplicationNameSubjectKind::Relation,
        3 => ApplicationNameSubjectKind::Group,
        _ => return Err(Denial::new(Kind::InvalidRecordShape)),
    };
    let subject_name = input.text()?.to_owned();
    let role_order = decode_sequence(input, budget, 2, |input, _| Ok(input.text()?.to_owned()))?;
    let role_dependencies = decode_sequence(input, budget, 2, |input, _| {
        let role = input.text()?.to_owned();
        let depends_on = input.text()?.to_owned();
        Ok(ApplicationNameRoleDependency::new(role, depends_on))
    })?;
    let split_capable = decode_bool(input)?;
    let split_discriminator_identity = if decode_bool(input)? {
        Some(input.text()?.to_owned())
    } else {
        None
    };
    let discriminator_referenced_roles =
        decode_sequence(input, budget, 2, |input, _| Ok(input.text()?.to_owned()))?;
    let require_authored_occurrence = decode_bool(input)?;
    Ok(ApplicationSchemaMember::ApplicationNameScheme {
        definition: ErasedApplicationNameScheme::from_untrusted_parts(
            scheme_id,
            subject_kind,
            subject_name,
            role_order,
            role_dependencies,
            split_capable,
            split_discriminator_identity,
            discriminator_referenced_roles,
            require_authored_occurrence,
        ),
    })
}

fn decode_bool(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
