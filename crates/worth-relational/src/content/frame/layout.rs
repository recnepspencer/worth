use super::super::{RelationalContentDenial as Denial, MAXIMUM_CONTENT_FRAME_BYTES};
use worth_foundational::facade::ContentFormat;

const MAGIC: &[u8; 8] = b"WRTHCONT";

pub(super) fn admit_length(observed: usize, format_maximum: usize) -> Result<(), Denial> {
    let maximum = format_maximum.min(MAXIMUM_CONTENT_FRAME_BYTES);
    if observed > maximum {
        return Err(Denial::SizeLimit { observed, maximum });
    }
    Ok(())
}

pub(super) fn header(format: &ContentFormat, length: usize) -> Result<Vec<u8>, Denial> {
    format.validate().map_err(|_| Denial::InvalidFormat)?;
    let mut header = Vec::with_capacity(24 + format.name().len());
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&1_u16.to_le_bytes());
    header.extend_from_slice(&(format.name().len() as u16).to_le_bytes());
    header.extend_from_slice(format.name().as_bytes());
    header.extend_from_slice(&format.version().to_le_bytes());
    header.extend_from_slice(&(length as u64).to_le_bytes());
    Ok(header)
}

pub(in crate::content) fn inspect(frame: &[u8]) -> Result<(&str, u32, usize), Denial> {
    let fixed = frame.get(..12).ok_or(Denial::NonCanonicalContent)?;
    if &fixed[..8] != MAGIC {
        return Err(Denial::NonCanonicalContent);
    }
    if fixed[8..10] != 1_u16.to_le_bytes() {
        return Err(Denial::UnsupportedVersion);
    }
    let name_len = u16::from_le_bytes([fixed[10], fixed[11]]) as usize;
    if !(1..=256).contains(&name_len) {
        return Err(Denial::InvalidFormat);
    }
    let offset = 24 + name_len;
    let name = frame
        .get(12..12 + name_len)
        .ok_or(Denial::NonCanonicalContent)?;
    let name = std::str::from_utf8(name).map_err(|_| Denial::InvalidFormat)?;
    let rest = frame
        .get(12 + name_len..offset)
        .ok_or(Denial::NonCanonicalContent)?;
    let version = u32::from_le_bytes(rest[..4].try_into().expect("fixed version width"));
    let payload = u64::from_le_bytes(rest[4..].try_into().expect("fixed length width"));
    let payload = usize::try_from(payload).map_err(|_| Denial::ArithmeticOverflow)?;
    if offset.checked_add(payload) != Some(frame.len()) {
        return Err(Denial::NonCanonicalContent);
    }
    ContentFormat::validate_parts(name, version).map_err(|_| Denial::InvalidFormat)?;
    Ok((name, version, offset))
}

pub(in crate::content) fn inspect_paid<'a>(
    frame: &'a [u8],
    control: &mut crate::content::RelationalContentControl<'_, '_, '_>,
) -> Result<(&'a str, u32, usize), Denial> {
    // The fixed envelope is24 bytes and the admitted ASCII name is at most256.
    control.checkpoint(frame.len().min(280) as u64)?;
    inspect(frame)
}
