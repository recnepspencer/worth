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

/// A zone whose only rule is the POSIX footer `rule`.
fn footer_only(rule: &str) -> WorthQueryTimeZone {
    WorthQueryTimeZone::from_tzif(&tzif(&[], &[0], rule)).unwrap()
}

#[test]
fn posix_rules_crossing_the_year_end_select_the_surrounding_transition() {
    let offset = |zone: &WorthQueryTimeZone, instant| zone.offset_at(instant).seconds_east();
    // Daylight time ends at 00:00 AEDT on Sunday 2023-01-01 (13:00Z).
    let january_end = footer_only("AEST-10AEDT,M10.1.0,M1.1.0/0");
    assert_eq!(offset(&january_end, at(2022, 12, 31, 12, 59)), 39_600);
    assert_eq!(offset(&january_end, at(2022, 12, 31, 13, 30)), 36_000);
    assert_eq!(
        january_end.date_at(at(2022, 12, 31, 13, 30)),
        Ok(date(2022, 12, 31))
    );
    // A negative start time: 22:00 EST on December 31 (03:00Z on January 1).
    let early_start = footer_only("EST5EDT,0/-2,M11.1.0");
    assert_eq!(offset(&early_start, at(2026, 1, 1, 2, 59)), -18_000);
    assert_eq!(offset(&early_start, at(2026, 1, 1, 3, 0)), -14_400);
    // An end past 24:00: 02:00 EDT on January 1 (06:00Z).
    let late_end = footer_only("EST5EDT,M3.2.0,J365/26");
    assert_eq!(offset(&late_end, at(2027, 1, 1, 5, 59)), -14_400);
    assert_eq!(offset(&late_end, at(2027, 1, 1, 6, 0)), -18_000);
    // Day 365 of a non-leap year is January 1 of the next.
    let day_365 = footer_only("EST5EDT,M3.2.0,365/0");
    assert_eq!(offset(&day_365, at(2027, 1, 1, 3, 59)), -14_400);
    assert_eq!(offset(&day_365, at(2027, 1, 1, 4, 0)), -18_000);
}

#[test]
fn tzif_footers_must_be_exactly_framed() {
    let malformed = Err(WorthQueryLocalCalendarDenial::MalformedZone);
    let mut truncated = tzif(&[], &[0], "MST7MDT,M3.2.0,M11.1.0");
    truncated.pop();
    assert_eq!(WorthQueryTimeZone::from_tzif(&truncated), malformed);
    let mut trailing = tzif(&[], &[0], "UTC0");
    trailing.extend_from_slice(b"garbage");
    assert_eq!(WorthQueryTimeZone::from_tzif(&trailing), malformed);
}

#[test]
fn tz_values_route_like_tzset_without_a_utc_fallback() {
    assert_eq!(
        WorthQueryTimeZone::from_tz_variable(""),
        Ok(WorthQueryTimeZone::UTC)
    );
    // `:` alone names the platform default, as an unset `TZ` does; never UTC.
    assert_eq!(
        WorthQueryTimeZone::from_tz_variable(":"),
        WorthQueryTimeZone::from_tz_environment(None)
    );
    #[cfg(not(windows))]
    assert_eq!(
        WorthQueryTimeZone::from_tz_environment(None),
        WorthQueryTimeZone::from_tz_variable("/etc/localtime")
    );
    assert_eq!(
        WorthQueryTimeZone::from_tz_environment(Some("EST5")),
        WorthQueryTimeZone::from_tz_variable("EST5")
    );
    // A bare value falls back to a POSIX rule; a `:` value is a file only.
    let eastern = WorthQueryTimeZone::from_tz_variable("EST5").unwrap();
    assert_eq!(
        eastern.offset_at(at(2026, 7, 1, 0, 0)).seconds_east(),
        -18_000
    );
    let not_found = Err(WorthQueryLocalCalendarDenial::SystemZoneUnreadable(
        std::io::ErrorKind::NotFound,
    ));
    assert_eq!(WorthQueryTimeZone::from_tz_variable(":EST5"), not_found);
    assert_eq!(WorthQueryTimeZone::from_tz_variable(":invalid"), not_found);
}

/// `reg query HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation`
/// output with the given values; names and types are never localized.
fn registry(
    bias: &str,
    daylight_bias: &str,
    standard: &str,
    daylight: &str,
    extra: &str,
) -> String {
    format!(
        "\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\TimeZoneInformation\r\n\
         \x20   ActiveTimeBias    REG_DWORD    {bias}\r\n\
         \x20   Bias    REG_DWORD    {bias}\r\n\
         \x20   DaylightBias    REG_DWORD    {daylight_bias}\r\n\
         \x20   DaylightName    REG_SZ    @tzres.dll,-191\r\n\
         \x20   DaylightStart    REG_BINARY    {daylight}\r\n\
         \x20   StandardBias    REG_DWORD    0x0\r\n\
         \x20   StandardName    REG_SZ    @tzres.dll,-192\r\n\
         \x20   StandardStart    REG_BINARY    {standard}\r\n\
         \x20   TimeZoneKeyName    REG_SZ    Some Standard Time\r\n{extra}\r\n"
    )
}

/// Mountain Standard Time: UTC-7, daylight from the second Sunday of March
/// 02:00 to the first Sunday of November 02:00.
fn mountain(extra: &str) -> String {
    registry(
        "0x1a4",
        "0xffffffc4",
        "00000B00000001000200000000000000",
        "00000300000002000200000000000000",
        extra,
    )
}

#[test]
fn a_windows_registry_zone_yields_its_yearly_rule_without_tz() {
    let zone = WorthQueryTimeZone::from_windows_registry_query(&mountain(
        "    DynamicDaylightTimeDisabled    REG_DWORD    0x0",
    ))
    .unwrap();
    let offset = |instant| zone.offset_at(instant).seconds_east();
    assert_eq!(offset(at(2026, 1, 15, 12, 0)), -25_200);
    assert_eq!(offset(at(2026, 7, 1, 12, 0)), -21_600);
    // 2026-03-08 02:00 MST is 09:00 UTC; 2026-11-01 02:00 MDT is 08:00 UTC.
    assert_eq!(offset(at(2026, 3, 8, 8, 59)), -25_200);
    assert_eq!(offset(at(2026, 3, 8, 9, 0)), -21_600);
    assert_eq!(offset(at(2026, 11, 1, 7, 59)), -21_600);
    assert_eq!(offset(at(2026, 11, 1, 8, 0)), -25_200);
    assert_eq!(zone.date_at(at(2026, 7, 2, 3, 0)), Ok(date(2026, 7, 1)));

    // Southern hemisphere: AUS Eastern, UTC+10, daylight October to April.
    let sydney = WorthQueryTimeZone::from_windows_registry_query(&registry(
        "0xfffffda8",
        "0xffffffc4",
        "00000400000001000300000000000000",
        "00000A00000001000200000000000000",
        "",
    ))
    .unwrap();
    assert_eq!(
        sydney.offset_at(at(2026, 1, 15, 0, 0)).seconds_east(),
        39_600
    );
    assert_eq!(
        sydney.offset_at(at(2026, 7, 1, 0, 0)).seconds_east(),
        36_000
    );
}

#[test]
fn a_windows_zone_without_daylight_time_is_fixed_and_a_malformed_record_refused() {
    let zeros = "00000000000000000000000000000000";
    let china = registry("0xfffffe20", "0xffffffc4", zeros, zeros, "");
    let china = WorthQueryTimeZone::from_windows_registry_query(&china).unwrap();
    for month in [1, 7] {
        assert_eq!(
            china.offset_at(at(2026, month, 1, 0, 0)).seconds_east(),
            28_800
        );
    }
    // Daylight adjustment switched off in Settings keeps standard time.
    let disabled = WorthQueryTimeZone::from_windows_registry_query(&mountain(
        "    DynamicDaylightTimeDisabled    REG_DWORD    0x1",
    ))
    .unwrap();
    assert_eq!(
        disabled.offset_at(at(2026, 7, 1, 12, 0)).seconds_east(),
        -25_200
    );

    let malformed = Err(WorthQueryLocalCalendarDenial::MalformedZone);
    let dated = registry(
        "0x1a4",
        "0xffffffc4",
        "EA070B00000001000200000000000000",
        "EA070300000002000200000000000000",
        "",
    );
    let bad_month = registry(
        "0x1a4",
        "0xffffffc4",
        "00000D00000001000200000000000000",
        "00000300000002000200000000000000",
        "",
    );
    for export in [
        dated,
        bad_month,
        mountain("").replace("Bias    REG_DWORD", "Bias    REG_SZ"),
    ] {
        assert_eq!(
            WorthQueryTimeZone::from_windows_registry_query(&export),
            malformed
        );
    }
    assert_eq!(
        WorthQueryTimeZone::from_windows_registry_query(""),
        malformed
    );
}
