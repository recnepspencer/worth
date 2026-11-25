use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryPortableExternalEffectContractParts, WorthQueryPortableExternalEffectContractRecord,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    effect: Option<&WorthQueryPortableExternalEffectContractRecord>,
) -> Result<(), Denial> {
    super::super::foundational_value::write_bool(output, effect.is_some())?;
    let Some(effect) = effect else {
        return Ok(());
    };
    output.text(effect.correlation_family().as_str())?;
    output.text(effect.effect())?;
    output.text(effect.payload_type().as_str())?;
    output.text(effect.protocol().identity().as_str())?;
    output.u32(effect.protocol().version().get())?;
    output.u64(effect.maximum_payload_bytes())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
) -> Result<Option<WorthQueryPortableExternalEffectContractRecord>, Denial> {
    if !super::super::foundational_value::decode_bool(input)? {
        return Ok(None);
    }
    let correlation_family = WorthQueryExternalEffectCorrelationFamily::new(input.text()?)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let effect = input.text()?.to_owned();
    let payload_type =
        WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, input.text()?.to_owned());
    let protocol_identity = BoundaryProtocolIdentity::parse(input.text()?.to_owned())
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let protocol_version = BoundaryProtocolVersion::try_new(input.u32()?)
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let maximum_payload_bytes = input.u64()?;
    Ok(Some(
        WorthQueryPortableExternalEffectContractRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableExternalEffectContractParts {
                correlation_family,
                effect,
                payload_type,
                protocol: ApplicationExternalEffectProtocol::new(
                    protocol_identity,
                    protocol_version,
                ),
                maximum_payload_bytes,
            },
        ),
    ))
}
