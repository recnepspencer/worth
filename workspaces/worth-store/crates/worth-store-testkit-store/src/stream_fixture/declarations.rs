pub use {
    worth_store::physical_runtime::RecordWriteSource,
    worth_store::physical_runtime::RecordWriteSourceError,
};

pub struct PatternSource {
    pub declared: u64,
    pub produced: u64,
    pub maximum_transfer: u64,
}

pub struct RepeatedByteSource {
    pub declared: u64,
    pub produced: u64,
    pub byte: u8,
}

impl RepeatedByteSource {
    pub const fn new(declared: u64, byte: u8) -> Self {
        Self {
            declared,
            produced: 0,
            byte,
        }
    }
}

impl RecordWriteSource for RepeatedByteSource {
    fn declared_length(&self) -> u64 {
        self.declared
    }

    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        let count = target.len().min((self.declared - self.produced) as usize);
        target[..count].fill(self.byte);
        self.produced += count as u64;
        Ok(count)
    }
}
impl PatternSource {
    pub const fn exact(bytes: u64) -> Self {
        Self {
            declared: bytes,
            produced: 0,
            maximum_transfer: u64::MAX,
        }
    }
    pub const fn fragmented(bytes: u64, maximum_transfer: u64) -> Self {
        assert!(
            maximum_transfer != 0,
            "a fragmented source must make progress"
        );
        Self {
            declared: bytes,
            produced: 0,
            maximum_transfer,
        }
    }
}
impl RecordWriteSource for PatternSource {
    fn declared_length(&self) -> u64 {
        self.declared
    }
    fn read_next(&mut self, target: &mut [u8]) -> Result<usize, RecordWriteSourceError> {
        if self.produced == self.declared {
            return Ok(0);
        }
        let count = target
            .len()
            .min((self.declared - self.produced) as usize)
            .min(self.maximum_transfer as usize);
        for (index, byte) in target[..count].iter_mut().enumerate() {
            *byte = pattern(self.produced + index as u64);
        }
        self.produced += count as u64;
        Ok(count)
    }
}

pub fn pattern(offset: u64) -> u8 {
    ((offset.wrapping_mul(31).wrapping_add(17)) & 0xff) as u8
}
pub fn pattern_digest(length: u64) -> u64 {
    (0..length).fold(0_u64, |digest, offset| {
        digest.rotate_left(5) ^ u64::from(pattern(offset))
    })
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
