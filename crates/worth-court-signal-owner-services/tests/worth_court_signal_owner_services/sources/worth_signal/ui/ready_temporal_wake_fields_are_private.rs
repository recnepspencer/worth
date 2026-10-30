use worth_signal::facade::{
    ClockTick, ReadyTemporalWake, TemporalCondition, TemporalWakeId, WakeOrdinal,
};

fn main() {
    let _wake = ReadyTemporalWake {
        id: TemporalWakeId::new(0),
        scheduled_ordinal: WakeOrdinal::new(1),
        ready_ordinal: WakeOrdinal::new(2),
        condition: TemporalCondition::after(5).unwrap(),
        due_tick: ClockTick::new(5),
        ready_tick: ClockTick::new(6),
    };
}
