//! Temporal declarations survive; clock leases and wake/runtime state do not.
use super::{Denial, Read, Write};
use crate::data::temporal::*;

pub(super) fn write(out: &mut Write<'_>, value: &TemporalCondition) -> Result<(), Denial> {
    let (tag, milliseconds) = match value {
        TemporalCondition::After(v) => (0, v.delay_ms()),
        TemporalCondition::AtOrAfter(v) => (1, v.tick().get()),
        TemporalCondition::Debounce(v) => (2, v.quiet_period_ms()),
        TemporalCondition::Throttle(v) => (3, v.window_ms()),
        TemporalCondition::StaleAfter(v) => (4, v.stale_after_ms()),
        TemporalCondition::Interval(v) => (5, v.period_ms()),
    };
    out.byte(tag)?;
    out.number(milliseconds)?;
    out.byte(match value.clock_domain() {
        ClockDomain::MonotonicExecution => 0,
        ClockDomain::WallClock => 1,
        ClockDomain::Presentation => 2,
    })?;
    if let TemporalCondition::Interval(value) = value {
        match value.anchor() {
            IntervalAnchor::Registration => out.byte(0)?,
            IntervalAnchor::FirstEvaluation => out.byte(1)?,
            IntervalAnchor::ExplicitTick(tick) => {
                out.byte(2)?;
                out.number(tick.get())?;
            }
        }
        out.byte(match value.missed_tick_policy() {
            MissedTickPolicy::CollapseToOne => 0,
            MissedTickPolicy::CatchUpAll => 1,
            MissedTickPolicy::SkipToLatest => 2,
        })?;
    }
    Ok(())
}
pub(super) fn read(read: &mut Read<'_>) -> Result<TemporalCondition, Denial> {
    let tag = read.byte()?;
    let milliseconds = read.number()?;
    let domain = match read.byte()? {
        0 => ClockDomain::MonotonicExecution,
        1 => ClockDomain::WallClock,
        2 => ClockDomain::Presentation,
        _ => return Err(Denial::Malformed),
    };
    let invalid = |_| Denial::Malformed;
    Ok(match tag {
        0 => TemporalCondition::After(
            AfterCondition::try_new(milliseconds)
                .map_err(invalid)?
                .with_clock_domain(domain)
                .map_err(invalid)?,
        ),
        1 => TemporalCondition::AtOrAfter(
            AtOrAfterCondition::new(ClockTick::new(milliseconds))
                .with_clock_domain(domain)
                .map_err(invalid)?,
        ),
        2 => TemporalCondition::Debounce(
            DebounceCondition::try_new(milliseconds)
                .map_err(invalid)?
                .with_clock_domain(domain)
                .map_err(invalid)?,
        ),
        3 => TemporalCondition::Throttle(
            ThrottleCondition::try_new(milliseconds)
                .map_err(invalid)?
                .with_clock_domain(domain)
                .map_err(invalid)?,
        ),
        4 => TemporalCondition::StaleAfter(
            StaleAfterCondition::try_new(milliseconds)
                .map_err(invalid)?
                .with_clock_domain(domain)
                .map_err(invalid)?,
        ),
        5 => {
            let anchor = match read.byte()? {
                0 => IntervalAnchor::Registration,
                1 => IntervalAnchor::FirstEvaluation,
                2 => IntervalAnchor::ExplicitTick(ClockTick::new(read.number()?)),
                _ => return Err(Denial::Malformed),
            };
            let missed = match read.byte()? {
                0 => MissedTickPolicy::CollapseToOne,
                1 => MissedTickPolicy::CatchUpAll,
                2 => MissedTickPolicy::SkipToLatest,
                _ => return Err(Denial::Malformed),
            };
            TemporalCondition::Interval(
                IntervalCondition::try_new(milliseconds)
                    .map_err(invalid)?
                    .with_anchor(anchor)
                    .with_missed_tick_policy(missed)
                    .with_clock_domain(domain)
                    .map_err(invalid)?,
            )
        }
        _ => return Err(Denial::Malformed),
    })
}
