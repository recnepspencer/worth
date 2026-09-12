use crate::PhysicalIsolationCounterSnapshot;

#[cfg(any(test, feature = "certification-authority"))]
pub(crate) fn foreground_reservation_test_progression_identity(
    counters: PhysicalIsolationCounterSnapshot,
) -> u64 {
    let mut identity = 0x5355_0000_0000_0002_u64;
    identity = mix_u64(identity, counters.outcome_count());
    identity = mix_u64(identity, counters.wait_count());
    identity = mix_u64(identity, counters.retry_count());
    identity = mix_u64(identity, counters.latch_counter_rows());
    identity = mix_u64(identity, counters.reclaim_counter_rows());
    mix_u64(identity, counters.protected_byte_footprint())
}

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
