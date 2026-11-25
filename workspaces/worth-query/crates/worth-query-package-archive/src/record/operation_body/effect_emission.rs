//! V43 carries a body's ordinary effect: its name and the input and payload
//! bindings whose types derive the payload. Older profiles have no slot.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyEffectEmission as Emission;

pub(super) const PROFILE: u16 = 43;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    emission: Option<&Emission>,
    version: u16,
) -> Result<(), Denial> {
    if version < PROFILE {
        return match emission {
            Some(_) => Err(Denial::new(Kind::UnsupportedRecordVariant)),
            None => Ok(()),
        };
    }
    output.u8(u8::from(emission.is_some()))?;
    if let Some(emission) = emission {
        output.text(&emission.effect)?;
        output.text(emission.input_binding.as_str())?;
        output.text(emission.payload_binding.as_str())?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    version: u16,
) -> Result<Option<Emission>, Denial> {
    if version < PROFILE {
        return Ok(None);
    }
    match input.u8()? {
        0 => Ok(None),
        1 => Ok(Some(Emission {
            effect: input.text()?.to_owned(),
            input_binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                input.text()?.to_owned(),
            ),
            payload_binding: WorthQueryPortableTypeIdentity::from_untrusted(
                ARCHIVE_DECODE,
                input.text()?.to_owned(),
            ),
        })),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}
