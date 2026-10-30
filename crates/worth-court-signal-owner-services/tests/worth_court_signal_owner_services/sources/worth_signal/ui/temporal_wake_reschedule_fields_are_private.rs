use worth_signal::facade::{RetiredTemporalWake, ScheduledTemporalWake, TemporalWakeReschedule};

fn main() {
    let retired: RetiredTemporalWake = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let scheduled: ScheduledTemporalWake =
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let _reschedule = TemporalWakeReschedule {
        retired,
        scheduled,
    };
}
