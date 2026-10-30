use worth_signal::facade::{
    ClockTick, ScheduledTemporalWake, TemporalCondition, TemporalWakeId, WakeOrdinal,
};

fn main() {
    let _wake = ScheduledTemporalWake {
        id: TemporalWakeId::new(0),
        ordinal: WakeOrdinal::new(1),
        condition: TemporalCondition::after(5).unwrap(),
        due_tick: ClockTick::new(5),
    };
}
