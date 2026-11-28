use std::num::NonZeroUsize;

use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, IntervalAnchor, IntervalCondition,
    MissedTickPolicy, TemporalCondition,
};

use super::super::original_lowering::source;
use worth_runtime_bridge::facade::BridgeTemporalCondition;

#[test]
fn all_six_conditions_schedule_on_the_signal_clock_and_interval_regenerates() {
    let bridge = source::runtime();
    let interval = IntervalCondition::try_new(4)
        .unwrap()
        .with_anchor(IntervalAnchor::ExplicitTick(ClockTick::new(7)))
        .with_missed_tick_policy(MissedTickPolicy::CatchUpAll);
    let conditions = [
        (TemporalCondition::after(2).unwrap(), 2),
        (TemporalCondition::at_or_after(ClockTick::new(3)), 3),
        (TemporalCondition::debounce(4).unwrap(), 4),
        (TemporalCondition::throttle(5).unwrap(), 5),
        (TemporalCondition::stale_after(6).unwrap(), 6),
        (TemporalCondition::interval(interval), 7),
    ];
    let wakes = conditions
        .iter()
        .map(|(condition, _)| {
            bridge
                .schedule_signal_wake(BridgeTemporalCondition::new(condition.clone()))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let advance = bridge
        .advance_signal_clock(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(20)),
            NonZeroUsize::new(6).unwrap(),
        )
        .unwrap();
    let ready = advance.promotion().ready_wakes();
    assert_eq!(ready.len(), conditions.len());
    for ((condition, due), wake) in conditions.iter().zip(ready) {
        assert_eq!(wake.condition(), condition);
        assert_eq!(wake.due_tick(), ClockTick::new(*due));
    }
    let (_, regeneration) = bridge
        .regenerate_signal_interval_wake(wakes.last().unwrap())
        .unwrap();
    assert_eq!(regeneration.scheduled().due_tick(), ClockTick::new(11));
    assert_eq!(regeneration.suppressed_interval_count(), 0);
    let compaction = bridge.compact_retired_signal_wakes(0, 1).unwrap();
    assert_eq!(compaction.expired_now(), 1);
}
