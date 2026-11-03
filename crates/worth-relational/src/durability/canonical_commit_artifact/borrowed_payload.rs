//! Allocation-free slicing of this artifact's MessagePack input for typed quotes.
use super::RelationalCanonicalCommitArtifactRefusal as Refusal;
use worth_execution::ExecutionRequest;
mod cursor;
use cursor::Cursor;

#[derive(Clone, Copy)]
pub(crate) struct BorrowedPayload<'a>(pub(crate) &'a [u8]);

impl<'a> BorrowedPayload<'a> {
    /// Frame-only traversal grants no decoded authority or request execution.
    /// Admission uses the controlled traversal below before creating owners.
    pub(crate) fn frame_fields(
        self,
        mut visit: impl FnMut(&'a str, Self) -> Result<(), Refusal>,
    ) -> Result<(), Refusal> {
        let mut cursor = Cursor::for_frame(self.0);
        let count = cursor.map_length()?;
        for _ in 0..count {
            let name = cursor.string()?;
            visit(name, Self(cursor.value()?))?;
        }
        cursor.finish()
    }
    pub(crate) fn field(
        self,
        name: &str,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut selected = None;
        self.fields(request, |field, raw| {
            if field == name && selected.replace(raw).is_some() {
                return Err(malformed());
            }
            Ok(())
        })?;
        selected.ok_or_else(malformed)
    }
    pub(crate) fn text_length(self, request: &ExecutionRequest<'_, '_>) -> Result<usize, Refusal> {
        Ok(self.text(request)?.len())
    }
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

    pub(crate) fn pairs(
        self,
        request: &ExecutionRequest<'_, '_>,
        mut visit: impl FnMut(Self, Self) -> Result<(), Refusal>,
    ) -> Result<usize, Refusal> {
        let mut cursor = Cursor::new(self.0, request);
        let count = cursor.map_length()?;
        for _ in 0..count {
            let key = Self(cursor.value()?);
            visit(key, Self(cursor.value()?))?;
        }
        cursor.finish()?;
        Ok(count)
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

    pub(crate) fn octets<'request, 'authority>(
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

    pub(crate) fn is_none(self) -> bool {
        self.0 == [0xc0]
    }
}

/// A borrowed Native binary subcodec, whether Serde emitted bin or a byte array.
/// No intermediate byte Vec is constructed by the allocation preflight.
pub(crate) struct Octets<'a, 'request, 'authority> {
    cursor: Cursor<'a, 'request, 'authority>,
    binary: bool,
    remaining: usize,
    count: usize,
}
impl Octets<'_, '_, '_> {
    pub(crate) fn len(&self) -> usize {
        self.count
    }
    pub(crate) fn byte(&mut self) -> Result<u8, Refusal> {
        self.cursor.check_live()?;
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
    pub(crate) fn skip(&mut self, count: usize) -> Result<(), Refusal> {
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
    pub(crate) fn finish(self) -> Result<(), Refusal> {
        if self.remaining != 0 {
            return Err(malformed());
        }
        self.cursor.finish()
    }
}

pub(crate) fn malformed() -> Refusal {
    Refusal::PayloadDecode {
        detail: "invalid canonical allocation-quote input".to_owned(),
    }
}

#[cfg(test)]
mod omission_for_test;
#[cfg(test)]
pub(super) use omission_for_test::{omit_nil_record_field, replace_record};
