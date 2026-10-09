use super::sink::CanonicalMaterialSink;
use std::fmt::Write;

struct ScalarToken {
    bytes: [u8; 40],
    length: usize,
}

impl Write for ScalarToken {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let end = self
            .length
            .checked_add(value.len())
            .ok_or(std::fmt::Error)?;
        self.bytes
            .get_mut(self.length..end)
            .ok_or(std::fmt::Error)?
            .copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}

fn format_scalar<S: CanonicalMaterialSink>(
    material: &mut S,
    value: impl std::fmt::Display,
    maximum_bytes: usize,
) -> Result<ScalarToken, S::Error> {
    material.admit_work(40 + maximum_bytes + 1)?;
    let mut token = ScalarToken {
        bytes: [0; 40],
        length: 0,
    };
    write!(&mut token, "{value}").expect("integer grammar fits its fixed 40-byte stack buffer");
    Ok(token)
}

impl ScalarToken {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.length]).expect("integer formatting is ASCII")
    }
}

pub(super) fn append_token<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: &str,
) -> Result<(), S::Error> {
    material.append(label)?;
    material.append("#")?;
    let length = format_scalar(material, value.len(), 20)?;
    material.append(length.as_str())?;
    material.append(":")?;
    material.append(value)?;
    material.append(";")
}

pub(super) fn append_bytes<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: &[u8],
) -> Result<(), S::Error> {
    material.append(label)?;
    material.append("#")?;
    let length = format_scalar(material, value.len(), 20)?;
    material.append(length.as_str())?;
    material.append(":")?;
    for byte in value {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        material.admit_work(2)?;
        let digits = [HEX[usize::from(*byte >> 4)], HEX[usize::from(*byte & 15)]];
        material.append(std::str::from_utf8(&digits).expect("hex digits are ASCII"))?;
    }
    material.append(";")
}

pub(super) fn append_u32<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: u32,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 10)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_u32_text<S: CanonicalMaterialSink>(
    material: &mut S,
    value: u32,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 10)?;
    material.append(value.as_str())
}

pub(super) fn append_u64<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: u64,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 20)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_i32<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: i32,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 11)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_i64<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: i64,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 20)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_i128<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: i128,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 40)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_u128<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    value: u128,
) -> Result<(), S::Error> {
    let value = format_scalar(material, value, 39)?;
    append_token(material, label, value.as_str())
}

pub(super) fn append_token_parts<S: CanonicalMaterialSink>(
    material: &mut S,
    label: &str,
    suffix: &str,
    parts: &[&str],
) -> Result<(), S::Error> {
    material.admit_work(parts.len())?;
    material.append(label)?;
    material.append(suffix)?;
    material.append("#")?;
    let length = parts
        .iter()
        .try_fold(0usize, |length, part| length.checked_add(part.len()))
        .ok_or_else(|| material.accounting_overflow())?;
    let length = format_scalar(material, length, 20)?;
    material.append(length.as_str())?;
    material.append(":")?;
    for part in parts {
        material.append(part)?;
    }
    material.append(";")
}
