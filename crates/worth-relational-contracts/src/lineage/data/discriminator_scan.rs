//! Structural parser for domain-tagged canonical discriminator byte streams.
//!
//! Mirrors `ApplicationNameDiscriminatorEncoder` exactly using an iterative
//! scope workstack (no recursion, no arbitrary depth limit). Extracts every
//! embedded 32-byte `TAG_NAME_REF` digest in exact encoding order and fails
//! closed on any non-empty payload that lacks the canonical v1 domain header,
//! carries non-canonical boolean/UTF-8 encodings, truncates early, or contains
//! trailing/unknown bytes.

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

struct Scope<'a> {
    cursor: Cursor<'a>,
    /// Number of value expressions still required before returning to item
    /// position in this scope (`0` = item position).
    pending_values: usize,
}

/// Scans a canonical discriminator payload and returns every embedded 32-byte
/// `TAG_NAME_REF` digest in encoding order.
///
/// Empty slices indicate no discriminator (`Ok(Vec::new())`). Any non-empty
/// slice MUST begin with `worth.query.name-discriminator.v1\0` and parse
/// completely to EOF; otherwise returns `Err(())`.
#[allow(
    clippy::result_unit_err,
    reason = "Callers classify malformed discriminators at their owning boundary"
)]
pub fn extract_embedded_name_refs(bytes: &[u8]) -> Result<Vec<[u8; 32]>, ()> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let rest = bytes
        .strip_prefix(NAME_DISCRIMINATOR_DOMAIN_TAG)
        .ok_or(())?;
    let mut root_cursor = Cursor(rest);
    root_cursor.read_len_prefixed_utf8()?;

    let mut refs = Vec::new();
    let mut stack = vec![Scope {
        cursor: root_cursor,
        pending_values: 0,
    }];

    while let Some(scope) = stack.last_mut() {
        if scope.pending_values == 0 {
            if scope.cursor.is_empty() {
                stack.pop();
                continue;
            }
            let tag = scope.cursor.read_u8()?;
            match tag {
                TAG_VARIANT => {
                    scope.cursor.read_len_prefixed_utf8()?;
                }
                TAG_FIELD => {
                    scope.cursor.read_len_prefixed_utf8()?;
                    scope.pending_values = 1;
                }
                other => {
                    if let Some(nested) = scan_value_tag(other, scope, &mut refs)? {
                        stack.push(nested);
                    }
                }
            }
        } else {
            scope.pending_values -= 1;
            let tag = scope.cursor.read_u8()?;
            if let Some(nested) = scan_value_tag(tag, scope, &mut refs)? {
                stack.push(nested);
            }
        }
    }

    Ok(refs)
}

fn scan_value_tag<'a>(
    tag: u8,
    scope: &mut Scope<'a>,
    refs: &mut Vec<[u8; 32]>,
) -> Result<Option<Scope<'a>>, ()> {
    match tag {
        TAG_BOOL => {
            let value = scope.cursor.read_u8()?;
            if value > 1 {
                return Err(());
            }
            Ok(None)
        }
        TAG_I8 | TAG_U8 => {
            scope.cursor.skip(1)?;
            Ok(None)
        }
        TAG_I16 | TAG_U16 => {
            scope.cursor.skip(2)?;
            Ok(None)
        }
        TAG_I32 | TAG_U32 => {
            scope.cursor.skip(4)?;
            Ok(None)
        }
        TAG_I64 | TAG_U64 => {
            scope.cursor.skip(8)?;
            Ok(None)
        }
        TAG_TEXT => {
            scope.cursor.read_len_prefixed_utf8()?;
            Ok(None)
        }
        TAG_UUID => {
            scope.cursor.skip(16)?;
            Ok(None)
        }
        TAG_NAME_REF => {
            let slice = scope.cursor.read_exact(32)?;
            let mut digest = [0u8; 32];
            digest.copy_from_slice(slice);
            refs.push(digest);
            Ok(None)
        }
        TAG_OPTION_NONE => Ok(None),
        TAG_OPTION_SOME => {
            scope.pending_values = scope.pending_values.checked_add(1).ok_or(())?;
            Ok(None)
        }
        TAG_PAIR => {
            scope.pending_values = scope.pending_values.checked_add(2).ok_or(())?;
            Ok(None)
        }
        TAG_NESTED_DISCRIMINATOR => {
            scope.cursor.read_len_prefixed_utf8()?;
            let nested_body = scope.cursor.read_len_prefixed_bytes()?;
            Ok(Some(Scope {
                cursor: Cursor(nested_body),
                pending_values: 0,
            }))
        }
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

    fn read_len_prefixed_bytes(&mut self) -> Result<&'a [u8], ()> {
        let len_bytes = self.read_exact(8)?;
        let len = u64::from_be_bytes(len_bytes.try_into().map_err(|_| ())?);
        let len = usize::try_from(len).map_err(|_| ())?;
        self.read_exact(len)
    }

    fn read_len_prefixed_utf8(&mut self) -> Result<&'a str, ()> {
        let slice = self.read_len_prefixed_bytes()?;
        std::str::from_utf8(slice).map_err(|_| ())
    }
}
