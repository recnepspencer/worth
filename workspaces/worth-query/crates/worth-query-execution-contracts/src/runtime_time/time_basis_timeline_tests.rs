//! Samples on the UTC-timestamp and local-date timelines: one trusted
//! reading, the calendar's day boundary, and refusal
//! without a calendar.
use std::sync::Arc;
use worth_execution::time::ControlledTime;

use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

use super::super::capability_calendar::WorthQueryCapabilityCalendarDenial;

use super::*;

const MICROS_PER_HOUR: i64 = 3_600_000_000;
const MICROS_PER_DAY: i64 = 24 * MICROS_PER_HOUR;

fn at(micros: i64) -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: micros,
    }
}

/// A fixed UTC-7 calendar written independently of the host zone engine.
fn utc_minus_seven(
    instant: CanonicalTimestamp,
) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial> {
    let local = instant.micros_since_unix_epoch - 7 * MICROS_PER_HOUR;
    i32::try_from(local.div_euclid(MICROS_PER_DAY))
        .map(|days_from_unix_epoch| CanonicalDate {
            days_from_unix_epoch,
        })
        .map_err(|_| WorthQueryCapabilityCalendarDenial)
}

fn calendared(source: Arc<ControlledTime>) -> WorthQueryRuntimeClock {
    WorthQueryRuntimeClock::from(source).with_capability_calendar(Arc::new(utc_minus_seven))
}

#[test]
fn utc_samples_preserve_microsecond_precision() {
    let micros = 1_795_829_400_000_123;
    let clock = WorthQueryRuntimeClock::from(Arc::new(ControlledTime::at_unix_nanos(
        u64::try_from(at(micros).micros_since_unix_epoch).unwrap() * 1_000,
    )));
    let sample = clock
        .sample(ApplicationCapabilityValidityTimeline::UtcTimestamp)
        .unwrap();
    assert_eq!(sample.value(), &AspectValue::Timestamp(at(micros)));
    assert_eq!(sample.instant(), at(micros));
}

#[test]
fn local_dates_turn_at_the_calendar_midnight_not_utc_midnight() {
    // 2026-11-27T06:59:59.999999Z is still 2026-11-26 at UTC-7.
    let day = 20_783;
    let local_midnight = (day + 1) * MICROS_PER_DAY + 7 * MICROS_PER_HOUR;
    let source = Arc::new(ControlledTime::at_unix_nanos(
        u64::try_from(at(local_midnight - 1).micros_since_unix_epoch).unwrap() * 1_000,
    ));
    let clock = calendared(source.clone());
    let date = |clock: &WorthQueryRuntimeClock| {
        clock
            .sample(ApplicationCapabilityValidityTimeline::LocalDate)
            .unwrap()
            .value()
            .clone()
    };
    assert_eq!(
        date(&clock),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: i32::try_from(day).unwrap(),
        })
    );
    source.set_wall_unix_nanos(
        u64::try_from(at(local_midnight).micros_since_unix_epoch).unwrap() * 1_000,
    );
    assert_eq!(
        date(&clock),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: i32::try_from(day + 1).unwrap(),
        })
    );
    assert!(clock.has_capability_calendar());
}

#[test]
fn local_dates_without_a_calendar_and_durations_on_civil_timelines_refuse() {
    let clock = WorthQueryRuntimeClock::from(Arc::new(ControlledTime::at_unix_nanos(
        u64::try_from(at(0).micros_since_unix_epoch).unwrap() * 1_000,
    )));
    assert!(!clock.has_capability_calendar());
    assert_eq!(
        clock.sample(ApplicationCapabilityValidityTimeline::LocalDate),
        Err(WorthQueryRuntimeTimeDenial::CalendarUnavailable)
    );
    for timeline in [
        ApplicationCapabilityValidityTimeline::UtcTimestamp,
        ApplicationCapabilityValidityTimeline::LocalDate,
    ] {
        assert!(matches!(
            calendared(Arc::new(ControlledTime::at_unix_nanos(
                u64::try_from(at(0).micros_since_unix_epoch).unwrap() * 1_000
            )))
            .sample_interval(timeline, std::time::Duration::from_secs(60)),
            Err(WorthQueryRuntimeTimeDenial::DurationNotRepresentable)
        ));
    }
}

#[test]
fn one_reading_samples_every_timeline_at_its_own_instant_after_the_source_moves() {
    let first = 20_783 * MICROS_PER_DAY + 3 * MICROS_PER_HOUR;
    let source = Arc::new(ControlledTime::at_unix_nanos(
        u64::try_from(at(first).micros_since_unix_epoch).unwrap() * 1_000,
    ));
    let clock = calendared(source.clone());
    let reading = clock.read().unwrap();
    source.set_wall_unix_nanos(
        u64::try_from(at(first + 2 * MICROS_PER_DAY).micros_since_unix_epoch).unwrap() * 1_000,
    );
    let utc = reading
        .sample(ApplicationCapabilityValidityTimeline::UtcTimestamp)
        .unwrap();
    let local = reading
        .sample(ApplicationCapabilityValidityTimeline::LocalDate)
        .unwrap();
    assert_eq!(utc.instant(), at(first));
    assert_eq!(local.instant(), at(first));
    // 03:00Z is still the previous local day at UTC-7.
    assert_eq!(
        local.value(),
        &AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_782,
        })
    );
    // A fresh sample reads the moved source.
    assert_eq!(
        clock
            .sample(ApplicationCapabilityValidityTimeline::UtcTimestamp)
            .unwrap()
            .instant(),
        at(first + 2 * MICROS_PER_DAY)
    );
}

#[test]
fn pre_epoch_readings_fail_closed_and_recover_after_wall_returns() {
    let source = Arc::new(ControlledTime::at_unix_secs(0));
    let clock = WorthQueryRuntimeClock::from(source.clone());
    source.step_wall_back(std::time::Duration::from_nanos(1));
    assert_eq!(
        clock.admission_timestamp(),
        Err(WorthQueryRuntimeTimeDenial::SourceUnavailable)
    );
    source.set_wall_unix_nanos(12_345_678_900);
    let reading = clock.read().unwrap();
    let seconds = reading
        .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
        .unwrap();
    let millis = reading
        .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
        .unwrap();
    assert_eq!(seconds.value(), &AspectValue::UInt64(12));
    assert_eq!(millis.value(), &AspectValue::UInt64(12_345));
}
