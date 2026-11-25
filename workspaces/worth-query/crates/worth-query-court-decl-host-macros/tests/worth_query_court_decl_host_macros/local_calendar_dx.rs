//! The host local calendar: a fixed offset or zone rules give the true local
//! day (never a UTC fallback), and the system zone agrees with UTC within a day.
use worth_query_decl::facade::application_schema::{CanonicalDate, CanonicalTimestamp};
use worth_query_host::facade::local_calendar::{
    WorthQueryLocalCalendarDenial, WorthQueryTimeZone, WorthQueryUtcOffset,
};

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}

fn date(year: i32, month: u8, day: u8) -> CanonicalDate {
    CanonicalDate::from_ymd(year, month, day).unwrap()
}

fn fixed(hours_east: i8) -> WorthQueryTimeZone {
    WorthQueryTimeZone::fixed(WorthQueryUtcOffset::from_hours_east(hours_east).unwrap())
}

/// A version 2 TZif file: an empty 32-bit block, then 64-bit transitions,
/// local time types (seconds east) and a footer rule.
fn tzif(transitions: &[(i64, u8)], offsets: &[i32], footer: &str) -> Vec<u8> {
    fn header(bytes: &mut Vec<u8>, timecnt: usize, typecnt: usize) {
        bytes.extend_from_slice(b"TZif2");
        bytes.extend_from_slice(&[0; 15]);
        for count in [0, 0, 0, timecnt, typecnt, 1] {
            bytes.extend_from_slice(&u32::try_from(count).unwrap().to_be_bytes());
        }
    }
    let mut bytes = Vec::new();
    header(&mut bytes, 0, 1);
    bytes.extend_from_slice(&[0; 7]);
    header(&mut bytes, transitions.len(), offsets.len());
    transitions
        .iter()
        .for_each(|(second, _)| bytes.extend(second.to_be_bytes()));
    transitions.iter().for_each(|&(_, kind)| bytes.push(kind));
    for offset in offsets {
        bytes.extend(offset.to_be_bytes());
        bytes.extend([0, 0]);
    }
    bytes.push(0);
    bytes.extend(format!("\n{footer}\n").bytes());
    bytes
}

fn second(instant: CanonicalTimestamp) -> i64 {
    instant.micros_since_unix_epoch / 1_000_000
}

#[test]
fn fixed_offsets_cross_midnight_leap_days_and_year_ends() {
    let utah = fixed(-7);
    let tokyo = fixed(9);
    assert_eq!(utah.date_at(at(2026, 11, 25, 1, 0)), Ok(date(2026, 11, 24)));
    assert_eq!(
        tokyo.date_at(at(2026, 11, 25, 16, 0)),
        Ok(date(2026, 11, 26))
    );
    assert_eq!(utah.date_at(at(2026, 11, 25, 7, 0)), Ok(date(2026, 11, 25)));
    assert_eq!(utah.date_at(at(2024, 3, 1, 5, 0)), Ok(date(2024, 2, 29)));
    assert_eq!(tokyo.date_at(at(2024, 2, 28, 16, 0)), Ok(date(2024, 2, 29)));
    assert_eq!(tokyo.date_at(at(2024, 2, 29, 15, 0)), Ok(date(2024, 3, 1)));
    assert_eq!(utah.date_at(at(2027, 1, 1, 3, 0)), Ok(date(2026, 12, 31)));
    assert_eq!(tokyo.date_at(at(2026, 12, 31, 15, 0)), Ok(date(2027, 1, 1)));
    assert_eq!(
        WorthQueryUtcOffset::from_seconds_east(-25_200)
            .unwrap()
            .date_at(at(2026, 11, 25, 1, 0)),
        Ok(date(2026, 11, 24))
    );
    assert_eq!(
        WorthQueryUtcOffset::from_seconds_east(93_600),
        Err(WorthQueryLocalCalendarDenial::OffsetOutOfRange)
    );
    let edge = CanonicalTimestamp {
        micros_since_unix_epoch: i64::MAX,
    };
    assert_eq!(
        tokyo.date_at(edge),
        Err(WorthQueryLocalCalendarDenial::InstantOutOfRange)
    );
}

#[test]
fn tzif_transitions_then_posix_footer_follow_daylight_time() {
    let (mst, mdt) = (-25_200, -21_600);
    let denver = WorthQueryTimeZone::from_tzif(&tzif(
        &[
            (second(at(2026, 3, 8, 9, 0)), 1),
            (second(at(2026, 11, 1, 8, 0)), 0),
        ],
        &[mst, mdt],
        "MST7MDT,M3.2.0,M11.1.0",
    ))
    .unwrap();
    let offset = |instant| denver.offset_at(instant).seconds_east();
    // Before the first transition, type 0 applies; then the transitions.
    assert_eq!(denver.date_at(at(2026, 1, 1, 6, 0)), Ok(date(2025, 12, 31)));
    assert_eq!(offset(at(2026, 3, 8, 8, 59)), mst);
    assert_eq!(offset(at(2026, 3, 8, 9, 0)), mdt);
    assert_eq!(denver.date_at(at(2026, 6, 1, 5, 0)), Ok(date(2026, 5, 31)));
    // After the last transition, the footer rule applies.
    assert_eq!(
        denver.date_at(at(2026, 11, 25, 1, 0)),
        Ok(date(2026, 11, 24))
    );
    assert_eq!(offset(at(2027, 3, 14, 8, 59)), mst);
    assert_eq!(offset(at(2027, 3, 14, 9, 0)), mdt);
    assert_eq!(denver.date_at(at(2027, 7, 1, 5, 30)), Ok(date(2027, 6, 30)));
    assert_eq!(offset(at(2027, 11, 7, 7, 59)), mdt);
    assert_eq!(offset(at(2027, 11, 7, 8, 0)), mst);
}

#[test]
fn southern_daylight_time_spans_the_year_end_and_malformed_rules_refuse() {
    let sydney =
        WorthQueryTimeZone::from_tzif(&tzif(&[], &[36_000], "AEST-10AEDT,M10.1.0,M4.1.0/3"))
            .unwrap();
    assert_eq!(
        sydney.offset_at(at(2026, 1, 15, 0, 0)).seconds_east(),
        39_600
    );
    assert_eq!(
        sydney.offset_at(at(2026, 7, 1, 0, 0)).seconds_east(),
        36_000
    );
    assert_eq!(
        sydney.date_at(at(2026, 12, 31, 13, 0)),
        Ok(date(2027, 1, 1))
    );
    let malformed = Err(WorthQueryLocalCalendarDenial::MalformedZone);
    assert_eq!(WorthQueryTimeZone::from_tzif(b"not a zone"), malformed);
    let mut truncated = tzif(&[(0, 0)], &[0], "UTC0");
    truncated.truncate(80);
    assert_eq!(WorthQueryTimeZone::from_tzif(&truncated), malformed);
    let bad_index = tzif(&[(0, 3)], &[0], "UTC0");
    assert_eq!(WorthQueryTimeZone::from_tzif(&bad_index), malformed);
    let bad_footer = tzif(&[], &[0], "MST7MDT,M13.1.0,M11.1.0");
    assert_eq!(WorthQueryTimeZone::from_tzif(&bad_footer), malformed);
}

#[test]
fn the_system_zone_today_is_within_a_day_of_the_utc_date() {
    let utc_today = WorthQueryTimeZone::UTC.today().unwrap();
    let local_today = match WorthQueryTimeZone::load_system() {
        Ok(zone) => zone.today().unwrap(),
        // A host without zone data refuses typed; it never falls back to UTC.
        Err(WorthQueryLocalCalendarDenial::SystemZoneUnreadable(_)) => return,
        Err(other) => panic!("system zone: {other}"),
    };
    let apart = local_today.days_from_unix_epoch - utc_today.days_from_unix_epoch;
    assert!(apart.abs() <= 1, "{local_today} vs UTC {utc_today}");
}
