use harfrust::{FontRef, Tag};
use sha2::{Digest, Sha256};

use super::UiQualifiedFontNameRecordReceipt;
use crate::font_collection::UiFontCollectionAdmissionDenial;

pub(super) fn derive(
    font: &FontRef<'_>,
) -> Result<
    (
        Box<[UiQualifiedFontNameRecordReceipt]>,
        Box<[UiQualifiedFontNameRecordReceipt]>,
    ),
    UiFontCollectionAdmissionDenial,
> {
    let data = font
        .table_data(Tag::from_be_bytes(*b"name"))
        .ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?;
    let bytes = data.as_bytes();
    let (count, strings) = header(bytes)?;
    let mut families = Vec::new();
    let mut styles = Vec::new();
    for index in 0..count {
        let Some(receipt) = record(bytes, strings, index)? else {
            continue;
        };
        if matches!(receipt.name_id, 1 | 16) {
            families.push(receipt);
        } else {
            styles.push(receipt);
        }
    }
    if families.is_empty() || styles.is_empty() {
        return Err(UiFontCollectionAdmissionDenial::FaceMetadataMismatch);
    }
    families.sort_by_key(key);
    styles.sort_by_key(key);
    families.dedup();
    styles.dedup();
    Ok((families.into_boxed_slice(), styles.into_boxed_slice()))
}

fn header(bytes: &[u8]) -> Result<(usize, usize), UiFontCollectionAdmissionDenial> {
    let format = be_u16(bytes, 0).ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?;
    let count =
        usize::from(be_u16(bytes, 2).ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?);
    let strings =
        usize::from(be_u16(bytes, 4).ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?);
    let records_end = 6usize
        .checked_add(
            count
                .checked_mul(12)
                .ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?,
        )
        .ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?;
    if format > 1 || records_end > bytes.len() || strings < records_end || strings > bytes.len() {
        return Err(UiFontCollectionAdmissionDenial::FaceMetadataMismatch);
    }
    Ok((count, strings))
}

fn record(
    bytes: &[u8],
    strings: usize,
    index: usize,
) -> Result<Option<UiQualifiedFontNameRecordReceipt>, UiFontCollectionAdmissionDenial> {
    let start = 6 + index * 12;
    let platform_id = field(bytes, start)?;
    let encoding_id = field(bytes, start + 2)?;
    let language_id = field(bytes, start + 4)?;
    let name_id = field(bytes, start + 6)?;
    let length = usize::from(field(bytes, start + 8)?);
    let offset = usize::from(field(bytes, start + 10)?);
    let end = strings
        .checked_add(offset)
        .and_then(|value| value.checked_add(length))
        .ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)?;
    if length == 0 || end > bytes.len() {
        return Err(UiFontCollectionAdmissionDenial::FaceMetadataMismatch);
    }
    if !matches!(name_id, 1 | 2 | 16 | 17) {
        return Ok(None);
    }
    let value = &bytes[strings + offset..end];
    let Some(valid) = valid_value(platform_id, value) else {
        return Ok(None);
    };
    if !valid {
        return Err(UiFontCollectionAdmissionDenial::FaceMetadataMismatch);
    }
    Ok(Some(UiQualifiedFontNameRecordReceipt {
        platform_id,
        encoding_id,
        language_id,
        name_id,
        content_digest: Sha256::digest(value).into(),
    }))
}

fn valid_value(platform_id: u16, value: &[u8]) -> Option<bool> {
    Some(match platform_id {
        0 | 3 => valid_utf16_name(value),
        1 => {
            value.iter().all(|byte| !byte.is_ascii_control())
                && value.iter().any(|byte| !byte.is_ascii_whitespace())
        }
        _ => return None,
    })
}

fn valid_utf16_name(value: &[u8]) -> bool {
    if !value.len().is_multiple_of(2) {
        return false;
    }
    let mut visible = false;
    for character in std::char::decode_utf16(
        value
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]])),
    ) {
        let Ok(character) = character else {
            return false;
        };
        if character.is_control() {
            return false;
        }
        visible |= !character.is_whitespace();
    }
    visible
}

fn key(record: &UiQualifiedFontNameRecordReceipt) -> (u16, u16, u16, u16, [u8; 32]) {
    (
        record.platform_id,
        record.encoding_id,
        record.language_id,
        record.name_id,
        record.content_digest,
    )
}

fn field(bytes: &[u8], start: usize) -> Result<u16, UiFontCollectionAdmissionDenial> {
    be_u16(bytes, start).ok_or(UiFontCollectionAdmissionDenial::FaceMetadataMismatch)
}

fn be_u16(bytes: &[u8], start: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(start..start + 2)?.try_into().ok()?,
    ))
}
