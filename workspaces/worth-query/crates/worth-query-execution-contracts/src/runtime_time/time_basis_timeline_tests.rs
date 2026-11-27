//! Samples on the UTC-timestamp and local-date timelines: one trusted
//! reading, pre-epoch instants, the calendar's day boundary, and refusal
//! without a calendar.
use std::sync::Arc;

use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

use super::super::capability_calendar::{
    WorthQueryCalendaredRuntimeTimeSource, WorthQueryCapabilityCalendarDenial,
};
use super::super::WorthQueryFixedRuntimeTimeSource;
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

fn calendared(source: WorthQueryFixedRuntimeTimeSource) -> WorthQueryRuntimeClock {
    WorthQueryRuntimeClock::from_source(WorthQueryCalendaredRuntimeTimeSource::new(
        source,
        Arc::new(utc_minus_seven),
    ))
}

#[test]
fn utc_samples_carry_the_one_reading_including_before_the_epoch() {
    for micros in [1_795_829_400_000_123, -86_400_000_001] {
        let clock =
            WorthQueryRuntimeClock::from_source(WorthQueryFixedRuntimeTimeSource::new(at(micros)));
        let sample = clock
            .sample(ApplicationCapabilityValidityTimeline::UtcTimestamp)
            .unwrap();
        assert_eq!(sample.value(), &AspectValue::Timestamp(at(micros)));
        assert_eq!(sample.instant(), at(micros));
    }
}

#[test]
fn local_dates_turn_at_the_calendar_midnight_not_utc_midnight() {
    // 2026-11-27T06:59:59.999999Z is still 2026-11-26 at UTC-7.
    let day = 20_783;
    let local_midnight = (day + 1) * MICROS_PER_DAY + 7 * MICROS_PER_HOUR;
    let source = WorthQueryFixedRuntimeTimeSource::new(at(local_midnight - 1));
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
    source.set(at(local_midnight));
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
    let clock = WorthQueryRuntimeClock::from_source(WorthQueryFixedRuntimeTimeSource::new(at(0)));
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
            calendared(WorthQueryFixedRuntimeTimeSource::new(at(0)))
                .sample_interval(timeline, std::time::Duration::from_secs(60)),
            Err(WorthQueryRuntimeTimeDenial::DurationNotRepresentable)
        ));
    }
}
