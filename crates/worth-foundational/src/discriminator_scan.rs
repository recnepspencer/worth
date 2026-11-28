//! Structural reference scanning for the canonical name discriminator wire format.

use std::fmt;

/// Domain prefix of a top-level canonical name discriminator.
pub const NAME_DISCRIMINATOR_DOMAIN_TAG: &[u8] = b"worth.query.name-discriminator.v1\0";

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

/// Structural failure in a canonical discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscriminatorScanErrorKind {
    InvalidHeader,
    UnexpectedEof,
    UnknownTag {
        found: u8,
    },
    NonCanonicalBool {
        found: u8,
    },
    InvalidUtf8,
    MalformedArity,
    InvalidNestedLength,
    /// Unrecognized suffix at an item boundary after a complete item.
    TrailingBytes {
        found: u8,
    },
}

/// A structural failure and its absolute byte offset, including in nested scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscriminatorScanError {
    pub kind: DiscriminatorScanErrorKind,
    pub offset: usize,
}

impl fmt::Display for DiscriminatorScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at discriminator byte {}",
            self.kind, self.offset
        )
    }
}

impl std::error::Error for DiscriminatorScanError {}

use DiscriminatorScanErrorKind as Kind;

/// Extracts 32-byte name references in wire order. Empty input denotes the
/// absence of a discriminator. A nonempty input includes the full header.
/// The v1 body is an open sequence of items, so any further complete item is
/// valid. Typed decoding additionally checks the expected identity and fields.
///
/// Time is O(input bytes), and additional space is O(nesting depth + references).
pub fn scan_name_discriminator_references(
    bytes: &[u8],
) -> Result<Vec<[u8; 32]>, DiscriminatorScanError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let Some(body) = bytes.strip_prefix(NAME_DISCRIMINATOR_DOMAIN_TAG) else {
        let offset = bytes
            .iter()
            .zip(NAME_DISCRIMINATOR_DOMAIN_TAG)
            .take_while(|(actual, expected)| actual == expected)
            .count();
        return Err(DiscriminatorScanError {
            kind: Kind::InvalidHeader,
            offset,
        });
    };
    let mut cursor = Cursor {
        remaining: body,
        offset: NAME_DISCRIMINATOR_DOMAIN_TAG.len(),
    };
    cursor.read_text()?;
    let mut scopes = vec![Scope::new(cursor)];
    let mut references = Vec::new();
    while let Some(scope) = scopes.last_mut() {
        if scope.cursor.remaining.is_empty() {
            if scope.pending_values != 0 {
                return Err(scope.cursor.error(Kind::MalformedArity));
            }
            scopes.pop();
        } else if let Some(nested) = scope.scan_item(&mut references)? {
            scopes.push(nested);
        }
    }
    Ok(references)
}

struct Scope<'a> {
    cursor: Cursor<'a>,
    pending_values: usize,
    completed_item: bool,
}

impl<'a> Scope<'a> {
    fn new(cursor: Cursor<'a>) -> Self {
        Self {
            cursor,
            pending_values: 0,
            completed_item: false,
        }
    }

    fn scan_item(
        &mut self,
        references: &mut Vec<[u8; 32]>,
    ) -> Result<Option<Self>, DiscriminatorScanError> {
        let offset = self.cursor.offset;
        let positional = self.pending_values == 0;
        let tag = self.cursor.read::<1>()?[0];
        if matches!(tag, TAG_VARIANT | TAG_FIELD) {
            if !positional {
                return Err(DiscriminatorScanError {
                    kind: Kind::MalformedArity,
                    offset,
                });
            }
            self.cursor.read_text()?;
            self.pending_values = usize::from(tag == TAG_FIELD);
            self.completed_item |= self.pending_values == 0;
            return Ok(None);
        }

        self.pending_values = self.pending_values.saturating_sub(1);
        let nested = match tag {
            TAG_BOOL => {
                let found = self.cursor.read::<1>()?[0];
                if found > 1 {
                    return Err(DiscriminatorScanError {
                        kind: Kind::NonCanonicalBool { found },
                        offset: offset + 1,
                    });
                }
                None
            }
            TAG_I8 | TAG_U8 => self.cursor.skip(1)?,
            TAG_I16 | TAG_U16 => self.cursor.skip(2)?,
            TAG_I32 | TAG_U32 => self.cursor.skip(4)?,
            TAG_I64 | TAG_U64 => self.cursor.skip(8)?,
            TAG_UUID => self.cursor.skip(16)?,
            TAG_TEXT => {
                self.cursor.read_text()?;
                None
            }
            TAG_NAME_REF => {
                references.push(self.cursor.read::<32>()?);
                None
            }
            TAG_OPTION_NONE => None,
            TAG_OPTION_SOME | TAG_PAIR => {
                // Pending children are bounded by the number of bytes consumed plus one.
                self.pending_values += if tag == TAG_PAIR { 2 } else { 1 };
                None
            }
            TAG_NESTED_DISCRIMINATOR => {
                self.cursor.read_text()?;
                Some(Self::new(self.cursor.read_nested()?))
            }
            found => {
                let kind = if positional && self.completed_item {
                    Kind::TrailingBytes { found }
                } else {
                    Kind::UnknownTag { found }
                };
                return Err(DiscriminatorScanError { kind, offset });
            }
        };
        self.completed_item |= self.pending_values == 0;
        Ok(nested)
    }
}

struct Cursor<'a> {
    remaining: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn error(&self, kind: Kind) -> DiscriminatorScanError {
        DiscriminatorScanError {
            kind,
            offset: self.offset,
        }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DiscriminatorScanError> {
        let (head, tail) = self
            .remaining
            .split_at_checked(len)
            .ok_or(DiscriminatorScanError {
                kind: Kind::UnexpectedEof,
                offset: self.offset + self.remaining.len(),
            })?;
        self.remaining = tail;
        self.offset += len;
        Ok(head)
    }

    fn read<const N: usize>(&mut self) -> Result<[u8; N], DiscriminatorScanError> {
        Ok(*self
            .take(N)?
            .first_chunk::<N>()
            .expect("take returned N bytes"))
    }

    fn read_text(&mut self) -> Result<(), DiscriminatorScanError> {
        let len = u64::from_be_bytes(self.read()?);
        let offset = self.offset;
        let len = usize::try_from(len).unwrap_or(usize::MAX);
        let text = self.take(len)?;
        std::str::from_utf8(text).map_err(|error| DiscriminatorScanError {
            kind: Kind::InvalidUtf8,
            offset: offset + error.valid_up_to(),
        })?;
        Ok(())
    }

    fn read_nested(&mut self) -> Result<Self, DiscriminatorScanError> {
        let error = self.error(Kind::InvalidNestedLength);
        let len = u64::from_be_bytes(self.read().map_err(|_| error)?);
        let len = usize::try_from(len).map_err(|_| error)?;
        let offset = self.offset;
        let remaining = self.take(len).map_err(|_| error)?;
        Ok(Self { remaining, offset })
    }

    fn skip(&mut self, len: usize) -> Result<Option<Scope<'a>>, DiscriminatorScanError> {
        self.take(len)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
