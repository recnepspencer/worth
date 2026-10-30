use worth_signal::facade::{ClockTick, TemporalFrontierSnapshot, TemporalWakeId, WakeOrdinal};

fn main() {
    let _snapshot = TemporalFrontierSnapshot {
        scheduled_frontier_width: 1,
        ready_frontier_width: 2,
        next_due_tick: Some(ClockTick::new(3)),
        next_due_wake_id: Some(TemporalWakeId::new(0)),
        next_due_wake_ordinal: Some(WakeOrdinal::new(1)),
        next_ready_wake_id: Some(TemporalWakeId::new(2)),
        next_ready_wake_ordinal: Some(WakeOrdinal::new(3)),
    };
}
