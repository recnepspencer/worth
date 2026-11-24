//! Allocation-free slicing of this artifact's MessagePack input for typed quotes.
use super::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
/// Traversal consumes interruption control, never execution construction authority.
pub trait PayloadCheckpoint {
    fn check_payload(
        &self,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause>;
}
impl PayloadCheckpoint for worth_execution::ExecutionRequest<'_, '_> {
    fn check_payload(
        &self,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
        self.check_live().map_err(Into::into)
    }
}
pub mod cursor;
pub use cursor::Cursor;

#[derive(Clone, Copy)]
pub struct BorrowedPayload<'a>(pub &'a [u8]);

impl<'a> BorrowedPayload<'a> {
    /// Frame-only traversal grants no decoded authority or request execution.
    /// Admission uses the controlled traversal below before creating owners.
    pub fn frame_fields(
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
    pub fn field(self, name: &str, request: &dyn PayloadCheckpoint) -> Result<Self, Refusal> {
        let mut selected = None;
        self.fields(request, |field, raw| {
            if field == name && selected.replace(raw).is_some() {
                return Err(malformed());
            }
            Ok(())
        })?;
        selected.ok_or_else(malformed)
    }
    pub fn text_length(self, request: &dyn PayloadCheckpoint) -> Result<usize, Refusal> {
        Ok(self.text(request)?.len())
    }
    pub fn fields(
        self,
        execution: &dyn PayloadCheckpoint,
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

    pub fn pairs(
        self,
        request: &dyn PayloadCheckpoint,
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

    pub fn elements(
        self,
        execution: &dyn PayloadCheckpoint,
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

    pub fn text(self, execution: &dyn PayloadCheckpoint) -> Result<&'a str, Refusal> {
        let mut cursor = Cursor::new(self.0, execution);
        let text = cursor.string()?;
        cursor.finish()?;
        Ok(text)
    }

    pub fn octets<'request>(
        self,
        execution: &'request dyn PayloadCheckpoint,
    ) -> Result<Octets<'a, 'request>, Refusal> {
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

    /// Only callers selecting actual Serde Option fields may default absence.
    pub fn optional_field(
        self,
        expected: &str,
        execution: &dyn PayloadCheckpoint,
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

    pub fn is_map(self) -> bool {
        matches!(self.0.first(), Some(0x80..=0x8f | 0xde..=0xdf))
    }

    pub fn is_none(self) -> bool {
        self.0 == [0xc0]
    }
}

/// A borrowed Native binary subcodec, whether Serde emitted bin or a byte array.
/// No intermediate byte Vec is constructed by the allocation preflight.
pub struct Octets<'a, 'request> {
    cursor: Cursor<'a, 'request>,
    binary: bool,
    remaining: usize,
    count: usize,
}
impl Octets<'_, '_> {
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn byte(&mut self) -> Result<u8, Refusal> {
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
    pub fn skip(&mut self, count: usize) -> Result<(), Refusal> {
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
    pub fn finish(self) -> Result<(), Refusal> {
        if self.remaining != 0 {
            return Err(malformed());
        }
        self.cursor.finish()
    }
}

pub fn malformed() -> Refusal {
    Refusal::PayloadDecode {
        detail: "invalid canonical allocation-quote input".to_owned(),
    }
}
