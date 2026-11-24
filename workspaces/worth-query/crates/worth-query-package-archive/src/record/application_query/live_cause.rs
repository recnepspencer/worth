use worth_query_declaration::facade::application_query::{
    ApplicationQueryLiveCauseContract, ApplicationQueryLiveTargetMode,
    WorthQueryPortableApplicationQueryLiveCauseParts,
};

use super::super::application_schema::wire_vocabulary::decode_type_identity;
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(super) fn write(
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
    })
}

pub(super) fn decode(
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
