#[cfg(any(test, feature = "certification-authority"))]
use super::PhysicalIsolationCounterSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutedIsolationBasis {
    executed_isolation_identity: u64,
    counter_identity: u64,
}

impl ExecutedIsolationBasis {
    #[cfg(any(test, feature = "certification-authority"))]
    pub(crate) const fn from_executed_isolation(
        executed_isolation_identity: u64,
        counters: PhysicalIsolationCounterSnapshot,
    ) -> Self {
        Self {
            executed_isolation_identity,
            counter_identity: counter_identity(counters),
        }
    }

    pub const fn executed_isolation_identity(self) -> u64 {
        self.executed_isolation_identity
    }

    pub const fn counter_identity(self) -> u64 {
        self.counter_identity
    }
}

#[cfg(any(test, feature = "certification-authority"))]
const fn counter_identity(counters: PhysicalIsolationCounterSnapshot) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    digest = mix_u64(digest, counters.outcome_count());
    digest = mix_u64(digest, counters.wait_count());
    digest = mix_u64(digest, counters.retry_count());
    digest = mix_u64(digest, counters.latch_counter_rows());
    digest = mix_u64(digest, counters.latch_wait_count());
    digest = mix_u64(digest, counters.reclaim_counter_rows());
    digest = mix_u64(digest, counters.blocked_maintenance_count());
    digest = mix_u64(digest, counters.reclaim_block_count());
    mix_u64(digest, counters.protected_byte_footprint())
}

#[cfg(any(test, feature = "certification-authority"))]
const fn mix_u64(mut digest: u64, value: u64) -> u64 {
    let bytes = value.to_le_bytes();
    let mut index = 0;
    while index < bytes.len() {
        digest ^= bytes[index] as u64;
        digest = digest.wrapping_mul(0x1000_0000_01b3);
        index += 1;
    }
    digest
}
