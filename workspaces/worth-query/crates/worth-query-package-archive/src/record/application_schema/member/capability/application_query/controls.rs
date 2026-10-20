use worth_query_declaration::facade::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryLaneEligibility,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::super::foundational_value;

pub(super) fn write_cardinality(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationQueryCardinality,
) -> Result<(), Denial> {
    output.u16(match value {
        ApplicationQueryCardinality::OptionalOne => 1,
        ApplicationQueryCardinality::ExactlyOne => 2,
        ApplicationQueryCardinality::Many => 3,
    })
}

pub(super) fn decode_cardinality(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryCardinality, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationQueryCardinality::OptionalOne),
        2 => Ok(ApplicationQueryCardinality::ExactlyOne),
        3 => Ok(ApplicationQueryCardinality::Many),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn write_authorization(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationQueryAuthorizationRequirement,
) -> Result<(), Denial> {
    match value {
        ApplicationQueryAuthorizationRequirement::Public => output.u16(1),
        ApplicationQueryAuthorizationRequirement::Ability {
            ability,
            scope_entity,
        } => {
            output.u16(2)?;
            output.text(ability)?;
            output.text(scope_entity)
        }
    }
}

pub(super) fn decode_authorization(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryAuthorizationRequirement, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationQueryAuthorizationRequirement::Public),
        2 => Ok(ApplicationQueryAuthorizationRequirement::Ability {
            ability: input.text()?.to_owned(),
            scope_entity: input.text()?.to_owned(),
        }),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(super) fn write_basis_support(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationQueryBasisSupport,
) -> Result<(), Denial> {
    foundational_value::write_bool(output, value.current())?;
    foundational_value::write_bool(output, value.pinned())?;
    foundational_value::write_bool(output, value.preview())
}

pub(super) fn decode_basis_support(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryBasisSupport, Denial> {
    let current = foundational_value::decode_bool(input)?;
    let pinned = foundational_value::decode_bool(input)?;
    let preview = foundational_value::decode_bool(input)?;
    if !current || !pinned {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let value = ApplicationQueryBasisSupport::current_and_pinned();
    Ok(if preview { value.with_preview() } else { value })
}

pub(super) fn write_lanes(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationQueryLaneEligibility,
) -> Result<(), Denial> {
    foundational_value::write_bool(output, value.one_shot_enabled())?;
    foundational_value::write_bool(output, value.historical_enabled())?;
    foundational_value::write_bool(output, value.live_enabled())?;
    foundational_value::write_bool(output, value.preview_enabled())
}

pub(super) fn decode_lanes(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationQueryLaneEligibility, Denial> {
    let one_shot = foundational_value::decode_bool(input)?;
    let historical = foundational_value::decode_bool(input)?;
    let live = foundational_value::decode_bool(input)?;
    let preview = foundational_value::decode_bool(input)?;
    if !one_shot {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let mut value = ApplicationQueryLaneEligibility::one_shot();
    if historical {
        value = value.with_historical();
    }
    if live {
        value = value.with_live();
    }
    if preview {
        value = value.with_preview();
    }
    Ok(value)
}
