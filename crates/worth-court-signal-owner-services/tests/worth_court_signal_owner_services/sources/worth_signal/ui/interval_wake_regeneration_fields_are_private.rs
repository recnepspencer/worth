use worth_signal::facade::{IntervalWakeRegeneration, RetiredTemporalWake, ScheduledTemporalWake};

fn main() {
    let retired: RetiredTemporalWake = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let scheduled: ScheduledTemporalWake =
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let _regeneration = IntervalWakeRegeneration {
        retired,
        scheduled,
        suppressed_interval_count: 2,
    };
}
