//! Structural parser for domain-tagged canonical discriminator byte streams.
//!
//! Extracts every embedded 32-byte `TAG_NAME_REF` digest from a
//! `worth.query.name-discriminator.v1\0` payload so Relational can verify that
//! no caller smuggles unverified name references inside opaque discriminator
//! bytes.

const NAME_DISCRIMINATOR_DOMAIN_TAG: &[u8] = b"worth.query.name-discriminator.v1\0";

const TAG_VARIANT: u8 = 0x01;
const TAG_FIELD: u8 = 0x02;
const TAG_BOOL: u8 = 0x10;
const TAG_I8: u8 = 0x11;
const TAG_I16: u8 = 0x12;
const TAG_I32: u8 = 0x13;
const TAG_I64: u8 = 0x14;
const TAG_U8: u8 = 0x15;
const TAG_U16: u8 = 0x16;
const TAG_U32: u8 = 0x17;
const TAG_U64: u8 = 0x18;
const TAG_TEXT: u8 = 0x19;
const TAG_UUID: u8 = 0x1a;
const TAG_NAME_REF: u8 = 0x20;
const TAG_OPTION_NONE: u8 = 0x30;
const TAG_OPTION_SOME: u8 = 0x31;
const TAG_PAIR: u8 = 0x32;
const TAG_NESTED_DISCRIMINATOR: u8 = 0x33;

/// Scans a canonical discriminator payload carrying the WORTH discriminator
/// domain header and returns every embedded 32-byte `TAG_NAME_REF` digest.
///
/// Non-domain-tagged opaque bytes (e.g. raw unit-test labels) return an empty
/// vector. Malformed domain-tagged streams return `Err(())`.
pub(super) fn extract_embedded_name_refs(bytes: &[u8]) -> Result<Vec<[u8; 32]>, ()> {
    let Some(rest) = bytes.strip_prefix(NAME_DISCRIMINATOR_DOMAIN_TAG) else {
        return Ok(Vec::new());
    };
    let mut cursor = Cursor(rest);
    cursor.skip_len_prefixed()?;
    let mut refs = Vec::new();
    while !cursor.is_empty() {
        scan_item(&mut cursor, &mut refs, 0)?;
    }
    Ok(refs)
}

fn scan_item(cursor: &mut Cursor<'_>, refs: &mut Vec<[u8; 32]>, depth: usize) -> Result<(), ()> {
    if depth > 64 {
        return Err(());
    }
    let tag = cursor.read_u8()?;
    match tag {
        TAG_VARIANT => cursor.skip_len_prefixed(),
        TAG_FIELD => {
            cursor.skip_len_prefixed()?;
            scan_value(cursor, refs, depth + 1)
        }
        other => scan_value_with_tag(other, cursor, refs, depth + 1),
    }
}

fn scan_value(cursor: &mut Cursor<'_>, refs: &mut Vec<[u8; 32]>, depth: usize) -> Result<(), ()> {
    if depth > 64 {
        return Err(());
    }
    let tag = cursor.read_u8()?;
    scan_value_with_tag(tag, cursor, refs, depth)
}

fn scan_value_with_tag(
    tag: u8,
    cursor: &mut Cursor<'_>,
    refs: &mut Vec<[u8; 32]>,
    depth: usize,
) -> Result<(), ()> {
    match tag {
        TAG_BOOL | TAG_I8 | TAG_U8 => cursor.skip(1),
        TAG_I16 | TAG_U16 => cursor.skip(2),
        TAG_I32 | TAG_U32 => cursor.skip(4),
        TAG_I64 | TAG_U64 => cursor.skip(8),
        TAG_TEXT => cursor.skip_len_prefixed(),
        TAG_UUID => cursor.skip(16),
        TAG_NAME_REF => {
            let slice = cursor.read_exact(32)?;
            let mut digest = [0u8; 32];
            digest.copy_from_slice(slice);
            refs.push(digest);
            Ok(())
        }
        TAG_OPTION_NONE => Ok(()),
        TAG_OPTION_SOME => scan_value(cursor, refs, depth + 1),
        TAG_PAIR => {
            scan_value(cursor, refs, depth + 1)?;
            scan_value(cursor, refs, depth + 1)
        }
        TAG_NESTED_DISCRIMINATOR => cursor.skip_len_prefixed(),
        _ => Err(()),
    }
}

struct Cursor<'a>(&'a [u8]);

impl<'a> Cursor<'a> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn read_u8(&mut self) -> Result<u8, ()> {
        let (&first, rest) = self.0.split_first().ok_or(())?;
        self.0 = rest;
        Ok(first)
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], ()> {
        if self.0.len() < len {
            return Err(());
        }
        let (head, tail) = self.0.split_at(len);
        self.0 = tail;
        Ok(head)
    }

    fn skip(&mut self, len: usize) -> Result<(), ()> {
        self.read_exact(len).map(|_| ())
    }

    fn skip_len_prefixed(&mut self) -> Result<(), ()> {
        let len_bytes = self.read_exact(8)?;
        let len = u64::from_be_bytes(len_bytes.try_into().map_err(|_| ())?);
        let len = usize::try_from(len).map_err(|_| ())?;
        self.skip(len)
    }
}
