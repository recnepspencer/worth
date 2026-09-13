#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalByteRange {
    offset: u64,
    length: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalByteRangeDenial {
    ZeroLength,
    EndOverflow,
}

impl PhysicalByteRange {
    pub const fn new(offset: u64, length: u64) -> Result<Self, PhysicalByteRangeDenial> {
        if length == 0 {
            return Err(PhysicalByteRangeDenial::ZeroLength);
        }
        if offset.checked_add(length).is_none() {
            return Err(PhysicalByteRangeDenial::EndOverflow);
        }
        Ok(Self { offset, length })
    }

    pub const fn offset(self) -> u64 {
        self.offset
    }

    pub const fn length(self) -> u64 {
        self.length
    }

    pub const fn end_exclusive(self) -> u64 {
        self.offset + self.length
    }
}

#[cfg(test)]
mod tests {
    use super::{PhysicalByteRange, PhysicalByteRangeDenial};

    #[test]
    fn range_rejects_empty_and_overflowing_descriptions() {
        assert_eq!(
            PhysicalByteRange::new(3, 0),
            Err(PhysicalByteRangeDenial::ZeroLength)
        );
        assert_eq!(
            PhysicalByteRange::new(u64::MAX, 1),
            Err(PhysicalByteRangeDenial::EndOverflow)
        );
        assert_eq!(PhysicalByteRange::new(3, 4).unwrap().end_exclusive(), 7);
    }
}
