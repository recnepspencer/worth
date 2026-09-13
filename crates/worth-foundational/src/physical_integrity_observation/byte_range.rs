use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalByteRange {
    offset: u64,
    length: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalByteRangeDenial {
    Empty,
    Overflow,
}

impl PhysicalByteRange {
    pub const fn new(offset: u64, length: u64) -> Result<Self, PhysicalByteRangeDenial> {
        if length == 0 {
            return Err(PhysicalByteRangeDenial::Empty);
        }
        if offset.checked_add(length).is_none() {
            return Err(PhysicalByteRangeDenial::Overflow);
        }
        Ok(Self { offset, length })
    }

    pub const fn offset(self) -> u64 {
        self.offset
    }

    pub const fn length(self) -> u64 {
        self.length
    }

    pub const fn end(self) -> u64 {
        self.offset + self.length
    }
}
