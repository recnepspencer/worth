//! Allocation-free slicing of this artifact's MessagePack input for typed quotes.
use super::RelationalCanonicalCommitArtifactRefusal as Refusal;
use worth_execution::ExecutionRequest;

#[derive(Clone, Copy)]
pub(crate) struct BorrowedPayload<'a>(pub(crate) &'a [u8]);

impl<'a> BorrowedPayload<'a> {
    pub(crate) fn fields(
        self,
        execution: &ExecutionRequest<'_, '_>,
        mut visit: impl FnMut(&'a str, Self) -> Result<(), Refusal>,
    ) -> Result<(), Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let count = cursor.map_length()?;
        for _ in 0..count {
            let name = cursor.string()?;
            let value = cursor.value()?;
            visit(name, Self(value))?;
        }
        cursor.finish()
    }

    pub(crate) fn elements(
        self,
        execution: &ExecutionRequest<'_, '_>,
        mut visit: impl FnMut(Self) -> Result<(), Refusal>,
    ) -> Result<usize, Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let count = cursor.array_length()?;
        for _ in 0..count {
            visit(Self(cursor.value()?))?;
        }
        cursor.finish()?;
        Ok(count)
    }

    pub(super) fn text(self, execution: &ExecutionRequest<'_, '_>) -> Result<&'a str, Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let text = cursor.string()?;
        cursor.finish()?;
        Ok(text)
    }

    pub(super) fn octets<'request, 'authority>(
        self,
        execution: &'request ExecutionRequest<'request, 'authority>,
    ) -> Result<Octets<'a, 'request, 'authority>, Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let marker = *self.0.first().ok_or_else(malformed)?;
        let binary = matches!(marker, 0xc4..=0xc6);
        let count = if binary {
            cursor.marker()?;
            cursor.length(match marker {
                0xc4 => 1,
                0xc5 => 2,
                _ => 4,
            })?
        } else {
            cursor.array_length()?
        };
        cursor.count(count)?;
        Ok(Octets {
            cursor,
            binary,
            remaining: count,
            count,
        })
    }

    /// Borrow one actual MessagePack bin allocation without staging a Vec.
    #[cfg(test)]
    pub(super) fn binary(self, execution: &ExecutionRequest<'_, '_>) -> Result<&'a [u8], Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let marker = cursor.marker()?;
        let width = match marker {
            0xc4 => 1,
            0xc5 => 2,
            0xc6 => 4,
            _ => return Err(malformed()),
        };
        let count = cursor.length(width)?;
        let bytes = cursor.take(count)?;
        cursor.finish()?;
        Ok(bytes)
    }

    /// Only callers selecting actual Serde Option fields may default absence.
    pub(super) fn optional_field(
        self,
        expected: &str,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut found = None;
        self.fields(execution, |name, value| {
            if name == expected && found.replace(value).is_some() {
                return Err(malformed());
            }
            Ok(())
        })?;
        Ok(found.unwrap_or(Self(&[0xc0])))
    }

    pub(super) fn is_map(self) -> bool {
        matches!(self.0.first(), Some(0x80..=0x8f | 0xde..=0xdf))
    }

    pub(super) fn is_none(self) -> bool {
        self.0 == [0xc0]
    }
}

/// A borrowed Native binary subcodec, whether Serde emitted bin or a byte array.
/// No intermediate byte Vec is constructed by the allocation preflight.
pub(super) struct Octets<'a, 'request, 'authority> {
    cursor: Cursor<'a, 'request, 'authority>,
    binary: bool,
    remaining: usize,
    count: usize,
}
impl Octets<'_, '_, '_> {
    pub(super) fn len(&self) -> usize {
        self.count
    }
    pub(super) fn remaining(&self) -> usize {
        self.remaining
    }
    pub(super) fn byte(&mut self) -> Result<u8, Refusal> {
        self.cursor
            .execution
            .check_live()
            .map_err(|stop| Refusal::Request(stop.into()))?;
        self.remaining = self.remaining.checked_sub(1).ok_or_else(malformed)?;
        let marker = self.cursor.marker()?;
        if self.binary || marker <= 0x7f {
            return Ok(marker);
        }
        if marker == 0xcc {
            self.cursor.marker()
        } else {
            Err(malformed())
        }
    }
    pub(super) fn u32(&mut self) -> Result<u32, Refusal> {
        Ok(u32::from_le_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }
    pub(super) fn skip(&mut self, count: usize) -> Result<(), Refusal> {
        if count > self.remaining {
            return Err(malformed());
        }
        if self.binary {
            self.cursor.take(count)?;
            self.remaining -= count;
        } else {
            for _ in 0..count {
                self.byte()?;
            }
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<(), Refusal> {
        if self.remaining != 0 {
            return Err(malformed());
        }
        self.cursor.finish()
    }
}

struct Cursor<'a, 'request, 'authority> {
    bytes: &'a [u8],
    next: usize,
    execution: &'request ExecutionRequest<'request, 'authority>,
}

impl<'a, 'request, 'authority> Cursor<'a, 'request, 'authority> {
    fn new(bytes: &'a [u8], execution: &'request ExecutionRequest<'request, 'authority>) -> Self {
        Self {
            bytes,
            next: 0,
            execution,
        }
    }
    fn finish(&self) -> Result<(), Refusal> {
        if self.next == self.bytes.len() {
            Ok(())
        } else {
            Err(malformed())
        }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8], Refusal> {
        let end = self.next.checked_add(length).ok_or_else(malformed)?;
        let bytes = self.bytes.get(self.next..end).ok_or_else(malformed)?;
        self.next = end;
        Ok(bytes)
    }
    fn marker(&mut self) -> Result<u8, Refusal> {
        Ok(self.take(1)?[0])
    }
    fn length(&mut self, width: usize) -> Result<usize, Refusal> {
        let bytes = self.take(width)?;
        let value = bytes
            .iter()
            .try_fold(0usize, |value, byte| {
                value.checked_mul(256)?.checked_add(usize::from(*byte))
            })
            .ok_or_else(malformed)?;
        Ok(value)
    }
    fn count(&self, count: usize) -> Result<usize, Refusal> {
        // Every array value/map item consumes at least one input byte.
        if count <= self.bytes.len() - self.next {
            Ok(count)
        } else {
            Err(malformed())
        }
    }
    fn map_length(&mut self) -> Result<usize, Refusal> {
        let count = match self.marker()? {
            marker @ 0x80..=0x8f => usize::from(marker & 15),
            0xde => self.length(2)?,
            0xdf => self.length(4)?,
            _ => return Err(malformed()),
        };
        self.count(count.checked_mul(2).ok_or_else(malformed)?)?;
        Ok(count)
    }
    fn array_length(&mut self) -> Result<usize, Refusal> {
        let count = match self.marker()? {
            marker @ 0x90..=0x9f => usize::from(marker & 15),
            0xdc => self.length(2)?,
            0xdd => self.length(4)?,
            _ => return Err(malformed()),
        };
        self.count(count)
    }
    fn string(&mut self) -> Result<&'a str, Refusal> {
        let length = match self.marker()? {
            marker @ 0xa0..=0xbf => usize::from(marker & 31),
            0xd9 => self.length(1)?,
            0xda => self.length(2)?,
            0xdb => self.length(4)?,
            _ => return Err(malformed()),
        };
        std::str::from_utf8(self.take(length)?).map_err(|_| malformed())
    }
    fn value(&mut self) -> Result<&'a [u8], Refusal> {
        let start = self.next;
        // This is a source-slice walker, not a decoded owner or a wire policy.
        // Its stack has the same 1024-depth ceiling as rmp-serde admission.
        let mut remaining = [0usize; 1024];
        let mut depth = 0;
        remaining[0] = 1;
        loop {
            self.execution
                .check_live()
                .map_err(|stop| Refusal::Request(stop.into()))?;
            if remaining[depth] == 0 {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                continue;
            }
            remaining[depth] -= 1;
            let marker = self.marker()?;
            let children = match marker {
                0x00..=0x7f | 0xc0 | 0xc2 | 0xc3 | 0xe0..=0xff => 0,
                0xcc | 0xd0 => {
                    self.take(1)?;
                    0
                }
                0xcd | 0xd1 => {
                    self.take(2)?;
                    0
                }
                0xce | 0xd2 | 0xca => {
                    self.take(4)?;
                    0
                }
                0xcf | 0xd3 | 0xcb => {
                    self.take(8)?;
                    0
                }
                0xa0..=0xbf => {
                    self.take(usize::from(marker & 31))?;
                    0
                }
                0xc4 | 0xd9 => {
                    let n = self.length(1)?;
                    self.take(n)?;
                    0
                }
                0xc5 | 0xda => {
                    let n = self.length(2)?;
                    self.take(n)?;
                    0
                }
                0xc6 | 0xdb => {
                    let n = self.length(4)?;
                    self.take(n)?;
                    0
                }
                // Ignored checkpoint fields may contain any MessagePack ext.
                // Include the signed tag byte in the checked borrowed skip;
                // the actual owner decoder still decides known-field meaning.
                0xc7..=0xc9 => {
                    let width = match marker {
                        0xc7 => 1,
                        0xc8 => 2,
                        _ => 4,
                    };
                    let n = self.length(width)?.checked_add(1).ok_or_else(malformed)?;
                    self.take(n)?;
                    0
                }
                0xd4..=0xd8 => {
                    self.take((1usize << (marker - 0xd4)) + 1)?;
                    0
                }
                0x90..=0x9f => usize::from(marker & 15),
                0x80..=0x8f => usize::from(marker & 15) * 2,
                0xdc => self.length(2)?,
                0xdd => self.length(4)?,
                0xde => self.length(2)?.checked_mul(2).ok_or_else(malformed)?,
                0xdf => self.length(4)?.checked_mul(2).ok_or_else(malformed)?,
                _ => return Err(malformed()),
            };
            if children != 0 {
                self.count(children)?;
                depth += 1;
                if depth == remaining.len() {
                    return Err(malformed());
                }
                remaining[depth] = children;
            }
        }
        Ok(&self.bytes[start..self.next])
    }
}

pub(super) fn malformed() -> Refusal {
    Refusal::PayloadDecode {
        detail: "invalid canonical allocation-quote input".to_owned(),
    }
}

#[cfg(test)]
mod omission_for_test;
#[cfg(test)]
pub(super) use omission_for_test::{omit_nil_record_field, replace_record};
