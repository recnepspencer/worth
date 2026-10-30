use worth_signal::facade::{
    ClockTick, SignalBranchId, TemporalPreviousValueAccess, TemporalWakeId, WakeOrdinal,
};

fn main() {
    let _access = TemporalPreviousValueAccess {
        branch_id: SignalBranchId(0),
        capability_epoch: 0,
        wake_id: TemporalWakeId::new(0),
        ready_ordinal: WakeOrdinal::new(1),
        ready_tick: ClockTick::new(2),
    };
}
