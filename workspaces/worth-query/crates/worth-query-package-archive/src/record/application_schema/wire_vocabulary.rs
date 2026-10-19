use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

pub(in crate::record) fn write_type_identity(
    output: &mut dyn BinaryEncodingSink,
    identity: &WorthQueryPortableTypeIdentity,
) -> Result<(), Denial> {
    output.text(identity.as_str())
}

pub(in crate::record) fn decode_type_identity(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryPortableTypeIdentity, Denial> {
    Ok(WorthQueryPortableTypeIdentity::from_untrusted(
        input.text()?.to_owned(),
    ))
}

pub(in crate::record) fn write_usize(
    output: &mut dyn BinaryEncodingSink,
    value: usize,
) -> Result<(), Denial> {
    let value = u64::try_from(value).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    output.u64(value)
}

pub(in crate::record) fn decode_usize(input: &mut BinaryInput<'_>) -> Result<usize, Denial> {
    usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::NumericWidthExceeded))
}

pub(in crate::record) fn write_optional<T: ?Sized>(
    output: &mut dyn BinaryEncodingSink,
    value: Option<&T>,
    write: impl FnOnce(&mut dyn BinaryEncodingSink, &T) -> Result<(), Denial>,
) -> Result<(), Denial> {
    match value {
        None => output.u16(0),
        Some(value) => {
            output.u16(1)?;
            write(output, value)
        }
    }
}

pub(in crate::record) fn decode_optional<T>(
    input: &mut BinaryInput<'_>,
    decode: impl FnOnce(&mut BinaryInput<'_>) -> Result<T, Denial>,
) -> Result<Option<T>, Denial> {
    match input.u16()? {
        0 => Ok(None),
        1 => decode(input).map(Some),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

pub(in crate::record) fn write_presence(
    output: &mut dyn BinaryEncodingSink,
    value: worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
) -> Result<(), Denial> {
    use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
    output.u16(match value {
        ApplicationFieldPresence::Required => 1,
        ApplicationFieldPresence::Optional => 2,
    })
}
pub(in crate::record) fn decode_presence(
    input: &mut BinaryInput<'_>,
) -> Result<worth_query_declaration::facade::application_schema::ApplicationFieldPresence, Denial> {
    use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
    match input.u16()? {
        1 => Ok(ApplicationFieldPresence::Required),
        2 => Ok(ApplicationFieldPresence::Optional),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
