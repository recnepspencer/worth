use super::*;
use crate::record::sequence::{decode_sequence, write_sequence};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationNameRoleDependency, ApplicationNameSubjectKind,
    ErasedApplicationGroupNameScheme, ErasedApplicationNameScheme,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationNameScheme,
) -> Result<(), Denial> {
    output.u16(1)?;
    write_scheme_fields(output, definition)
}

pub(super) fn write_group(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationGroupNameScheme,
) -> Result<(), Denial> {
    output.u16(1)?;
    write_scheme_fields(output, definition.name_scheme())?;
    output.u8(definition.continuity_rule().canonical_tag())?;
    write_optional_text(output, definition.anchor_role())?;
    write_optional_text(output, definition.boundary_role())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    budget.require_nesting_depth(2)?;
    Ok(ApplicationSchemaMember::ApplicationNameScheme {
        definition: decode_scheme_fields(input, budget)?,
    })
}

pub(super) fn decode_group(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    if input.u16()? != 1 {
        return Err(Denial::new(Kind::UnsupportedRecordVersion));
    }
    budget.require_nesting_depth(2)?;
    let name_scheme = decode_scheme_fields(input, budget)?;
    let continuity_rule = match input.u8()? {
        1 => ApplicationGroupContinuityRule::Anchor,
        2 => ApplicationGroupContinuityRule::Boundary,
        3 => ApplicationGroupContinuityRule::ProducerRole,
        _ => return Err(Denial::new(Kind::InvalidRecordShape)),
    };
    let anchor_role = decode_optional_text(input)?;
    let boundary_role = decode_optional_text(input)?;
    Ok(ApplicationSchemaMember::ApplicationGroupNameScheme {
        definition: ErasedApplicationGroupNameScheme::from_untrusted_parts(
            ARCHIVE_DECODE,
            name_scheme,
            continuity_rule,
            anchor_role,
            boundary_role,
        ),
    })
}

fn write_scheme_fields(
    output: &mut dyn BinaryEncodingSink,
    definition: &ErasedApplicationNameScheme,
) -> Result<(), Denial> {
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
    write_optional_text(output, definition.split_discriminator_identity())?;
    write_sequence(
        output,
        definition.discriminator_referenced_roles(),
        |output, role| output.text(role),
    )?;
    output.u8(u8::from(definition.require_authored_occurrence()))
}

fn decode_scheme_fields(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ErasedApplicationNameScheme, Denial> {
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
    let split_discriminator_identity = decode_optional_text(input)?;
    let discriminator_referenced_roles =
        decode_sequence(input, budget, 2, |input, _| Ok(input.text()?.to_owned()))?;
    let require_authored_occurrence = decode_bool(input)?;
    Ok(ErasedApplicationNameScheme::from_untrusted_parts(
        ARCHIVE_DECODE,
        scheme_id,
        subject_kind,
        subject_name,
        role_order,
        role_dependencies,
        split_capable,
        split_discriminator_identity,
        discriminator_referenced_roles,
        require_authored_occurrence,
    ))
}

fn write_optional_text(
    output: &mut dyn BinaryEncodingSink,
    value: Option<&str>,
) -> Result<(), Denial> {
    match value {
        Some(text) => {
            output.u8(1)?;
            output.text(text)
        }
        None => output.u8(0),
    }
}

fn decode_optional_text(input: &mut BinaryInput<'_>) -> Result<Option<String>, Denial> {
    if decode_bool(input)? {
        Ok(Some(input.text()?.to_owned()))
    } else {
        Ok(None)
    }
}

fn decode_bool(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
