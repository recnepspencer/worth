use super::ScheduleReplayDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchedulePerturbationSeed(u64);

impl SchedulePerturbationSeed {
    pub const fn from_u64(seed: u64) -> Self {
        Self(seed)
    }

    pub fn required(seed: Option<u64>) -> Result<Self, ScheduleReplayDenial> {
        seed.map(Self::from_u64)
            .ok_or(ScheduleReplayDenial::MissingSeed)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
