//! Bounded allocation-free MessagePack value traversal.
use super::{malformed, Refusal};
use worth_execution::ExecutionRequest;

pub(super) struct Cursor<'a, 'request, 'authority> {
    bytes: &'a [u8],
    next: usize,
    execution: Option<&'request ExecutionRequest<'request, 'authority>>,
}

impl<'a, 'request, 'authority> Cursor<'a, 'request, 'authority> {
    pub(super) fn new(
        bytes: &'a [u8],
        execution: &'request ExecutionRequest<'request, 'authority>,
    ) -> Self {
        Self {
            bytes,
            next: 0,
            execution: Some(execution),
        }
    }
    pub(super) fn for_frame(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            next: 0,
            execution: None,
        }
    }
    pub(super) fn check_live(&self) -> Result<(), Refusal> {
        if let Some(execution) = self.execution {
            execution
                .check_live()
                .map_err(|stop| Refusal::Request(stop.into()))?;
        }
        Ok(())
    }
    pub(super) fn finish(&self) -> Result<(), Refusal> {
        if self.next == self.bytes.len() {
            Ok(())
        } else {
            Err(malformed())
        }
    }
    pub(super) fn take(&mut self, length: usize) -> Result<&'a [u8], Refusal> {
        let end = self.next.checked_add(length).ok_or_else(malformed)?;
        let bytes = self.bytes.get(self.next..end).ok_or_else(malformed)?;
        self.next = end;
        Ok(bytes)
    }
    pub(super) fn marker(&mut self) -> Result<u8, Refusal> {
        Ok(self.take(1)?[0])
    }
    pub(super) fn length(&mut self, width: usize) -> Result<usize, Refusal> {
        let bytes = self.take(width)?;
        let value = bytes
            .iter()
            .try_fold(0usize, |value, byte| {
                value.checked_mul(256)?.checked_add(usize::from(*byte))
            })
            .ok_or_else(malformed)?;
        Ok(value)
    }
    pub(super) fn count(&self, count: usize) -> Result<usize, Refusal> {
        // Every array value/map item consumes at least one input byte.
        if count <= self.bytes.len() - self.next {
            Ok(count)
        } else {
            Err(malformed())
        }
    }
    pub(super) fn map_length(&mut self) -> Result<usize, Refusal> {
        let count = match self.marker()? {
            marker @ 0x80..=0x8f => usize::from(marker & 15),
            0xde => self.length(2)?,
            0xdf => self.length(4)?,
            _ => return Err(malformed()),
        };
        self.count(count.checked_mul(2).ok_or_else(malformed)?)?;
        Ok(count)
    }
    pub(super) fn array_length(&mut self) -> Result<usize, Refusal> {
        let count = match self.marker()? {
            marker @ 0x90..=0x9f => usize::from(marker & 15),
            0xdc => self.length(2)?,
            0xdd => self.length(4)?,
            _ => return Err(malformed()),
        };
        self.count(count)
    }
    pub(super) fn string(&mut self) -> Result<&'a str, Refusal> {
        let length = match self.marker()? {
            marker @ 0xa0..=0xbf => usize::from(marker & 31),
            0xd9 => self.length(1)?,
            0xda => self.length(2)?,
            0xdb => self.length(4)?,
            _ => return Err(malformed()),
        };
        std::str::from_utf8(self.take(length)?).map_err(|_| malformed())
    }
    pub(super) fn value(&mut self) -> Result<&'a [u8], Refusal> {
        let start = self.next;
        // This is a source-slice walker, not a decoded owner or a wire policy.
        // Its stack has the same 1024-depth ceiling as rmp-serde admission.
        let mut remaining = [0usize; 1024];
        let mut depth = 0;
        remaining[0] = 1;
        loop {
            self.check_live()?;
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
