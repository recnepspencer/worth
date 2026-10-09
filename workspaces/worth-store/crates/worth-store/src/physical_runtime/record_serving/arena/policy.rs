/// Maximum allocated range end within one arena file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtentArenaCapacity(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaEvacuationThreshold(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentArenaPolicyDenial {
    CapacityOutsideSupportedRange,
    InvalidEvacuationThreshold,
}

impl ExtentArenaCapacity {
    pub const DEFAULT: Self = Self(1 << 30);

    pub fn bytes(bytes: u64) -> Result<Self, ExtentArenaPolicyDenial> {
        if !(64 << 20..=4 << 30).contains(&bytes) {
            return Err(ExtentArenaPolicyDenial::CapacityOutsideSupportedRange);
        }
        Ok(Self(bytes))
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl ArenaEvacuationThreshold {
    pub const DEFAULT: Self = Self(50);

    pub fn percent(percent: u8) -> Result<Self, ExtentArenaPolicyDenial> {
        if !(1..100).contains(&percent) {
            return Err(ExtentArenaPolicyDenial::InvalidEvacuationThreshold);
        }
        Ok(Self(percent))
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub(in crate::physical_runtime::record_serving) fn requires_evacuation(
        self,
        live_bytes: u64,
        allocated_bytes: u64,
    ) -> bool {
        u128::from(live_bytes) * 100 < u128::from(allocated_bytes) * u128::from(self.0)
    }
}
