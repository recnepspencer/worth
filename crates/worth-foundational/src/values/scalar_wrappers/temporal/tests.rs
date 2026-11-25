use super::*;

#[test]
fn civil_date_uses_the_proleptic_gregorian_calendar() {
    for (text, ymd, days) in [
        ("0000-01-01", (0, 1, 1), -719_528),
        ("0000-02-29", (0, 2, 29), -719_469),
        ("1900-02-28", (1900, 2, 28), -25_509),
        ("1900-03-01", (1900, 3, 1), -25_508),
        ("1969-12-31", (1969, 12, 31), -1),
        ("1970-01-01", (1970, 1, 1), 0),
        ("1970-01-02", (1970, 1, 2), 1),
        ("2000-02-29", (2000, 2, 29), 11_016),
        ("2024-02-29", (2024, 2, 29), 19_782),
        ("2026-02-28", (2026, 2, 28), 20_512),
        ("2100-02-28", (2100, 2, 28), 47_540),
        ("2100-03-01", (2100, 3, 1), 47_541),
        ("9999-12-31", (9999, 12, 31), 2_932_896),
    ] {
        let date = CanonicalDate {
            days_from_unix_epoch: days,
        };
        assert_eq!(CanonicalDate::from_ymd(ymd.0, ymd.1, ymd.2), Ok(date));
        assert_eq!(date.to_ymd(), ymd);
        assert_eq!(text.parse(), Ok(date));
        assert_eq!(date.to_string(), text);
    }
}

#[test]
fn civil_date_rejects_invalid_dates_and_handles_negative_and_endpoint_years() {
    for (year, month, day) in [
        (1900, 2, 29),
        (2000, 2, 30),
        (2023, 2, 29),
        (2100, 2, 29),
        (-1, 2, 29),
        (-100, 2, 29),
        (2024, 0, 1),
        (2024, 13, 1),
        (2024, 1, 0),
        (2024, 4, 31),
        (2024, 6, 31),
        (2024, 9, 31),
        (2024, 11, 31),
        (2024, 12, 32),
    ] {
        assert_eq!(
            CanonicalDate::from_ymd(year, month, day),
            Err(CanonicalTemporalTextDenial::InvalidDate),
            "expected InvalidDate for ({year}, {month}, {day})"
        );
    }

    for text in [
        "1900-02-29",
        "2000-02-30",
        "2026-13-01",
        "20260228",
        "2026-02-28Z",
        "+002026-01-01",
        "-000000-01-01",
        "+01000-01-01",
        "-9999999999-xx-01",
    ] {
        assert_eq!(
            text.parse::<CanonicalDate>(),
            Err(CanonicalTemporalTextDenial::InvalidDate),
            "expected InvalidDate for {text:?}"
        );
    }

    for (year, leap_ok) in [(-4, true), (-100, false), (-400, true), (0, true)] {
        assert_eq!(CanonicalDate::from_ymd(year, 2, 29).is_ok(), leap_ok);
    }

    for year in [i32::MIN, i32::MAX, 6_000_000, -6_000_000] {
        assert_eq!(
            CanonicalDate::from_ymd(year, 1, 1),
            Err(CanonicalTemporalTextDenial::OutOfRange)
        );
    }
    assert_eq!(
        "+9999999999-01-01".parse::<CanonicalDate>(),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );

    for days_from_unix_epoch in [i32::MIN, i32::MIN + 1, -719_529, 0, i32::MAX - 1, i32::MAX] {
        let date = CanonicalDate {
            days_from_unix_epoch,
        };
        let (y, m, d) = date.to_ymd();
        assert_eq!(CanonicalDate::from_ymd(y, m, d), Ok(date));
        assert_eq!(date.to_string().parse(), Ok(date));
    }
}

#[test]
fn timestamp_civil_conversion_and_rfc3339_handle_offsets_fractions_and_endpoints() {
    let epoch = CanonicalTimestamp {
        micros_since_unix_epoch: 0,
    };
    assert_eq!(
        CanonicalTimestamp::from_ymd_hms_micros(1970, 1, 1, 0, 0, 0, 0),
        Ok(epoch)
    );
    assert_eq!(epoch.to_ymd_hms_micros(), (1970, 1, 1, 0, 0, 0, 0));
    assert_eq!(epoch.to_rfc3339(), Ok("1970-01-01T00:00:00Z".to_owned()));
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1970-01-01t00:00:00z"),
        Ok(epoch)
    );
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1970-01-01T00:00:00-00:00"),
        Ok(epoch)
    );

    let before_epoch = CanonicalTimestamp {
        micros_since_unix_epoch: -1,
    };
    assert_eq!(
        CanonicalTimestamp::from_ymd_hms_micros(1969, 12, 31, 23, 59, 59, 999_999),
        Ok(before_epoch)
    );
    assert_eq!(
        before_epoch.to_ymd_hms_micros(),
        (1969, 12, 31, 23, 59, 59, 999_999)
    );
    assert_eq!(
        before_epoch.to_rfc3339(),
        Ok("1969-12-31T23:59:59.999999Z".to_owned())
    );
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1970-01-01T00:59:59.999999+01:00"),
        Ok(before_epoch)
    );
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1969-12-31T22:59:59.999999000-01:00"),
        Ok(before_epoch)
    );

    let half_second = CanonicalTimestamp::from_ymd_hms_micros(2024, 2, 29, 12, 30, 45, 500_000)
        .expect("leap-day timestamp is valid");
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("2024-02-29T12:30:45.5Z"),
        Ok(half_second)
    );
    assert_eq!(
        half_second.to_rfc3339(),
        Ok("2024-02-29T12:30:45.500000Z".to_owned())
    );
    assert_eq!(
        half_second.utc_date(),
        CanonicalDate::from_ymd(2024, 2, 29).unwrap()
    );

    for invalid_components in [
        (2024, 2, 29, 24, 0, 0, 0),
        (2024, 2, 29, 0, 60, 0, 0),
        (2024, 2, 29, 0, 0, 60, 0),
        (2024, 2, 29, 0, 0, 0, 1_000_000),
    ] {
        assert_eq!(
            CanonicalTimestamp::from_ymd_hms_micros(
                invalid_components.0,
                invalid_components.1,
                invalid_components.2,
                invalid_components.3,
                invalid_components.4,
                invalid_components.5,
                invalid_components.6,
            ),
            Err(CanonicalTemporalTextDenial::InvalidTime)
        );
    }
    assert_eq!(
        CanonicalTimestamp::from_ymd_hms_micros(1900, 2, 29, 0, 0, 0, 0),
        Err(CanonicalTemporalTextDenial::InvalidDate)
    );

    for invalid in [
        "1970-01-01T00:00:00",
        "1970-01-01T00:00:60Z",
        "1970-01-01T00:00:00.Z",
        "1970-01-01T00:00:00.1234567Z",
        "1970-01-01T24:00:00Z",
        "1970-01-01T00:00:00+24:00",
        "1970-01-01T00:00:00+04:60",
        "1970-01-01T00:00:00Zextra",
        "1970-01-01T00:00:00+01:00Z",
    ] {
        assert_eq!(
            CanonicalTimestamp::from_rfc3339(invalid),
            Err(CanonicalTemporalTextDenial::InvalidTime),
            "expected InvalidTime for {invalid:?}"
        );
    }
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("+010000-01-01T00:00:00Z"),
        Err(CanonicalTemporalTextDenial::InvalidDate)
    );

    for micros_since_unix_epoch in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
        let timestamp = CanonicalTimestamp {
            micros_since_unix_epoch,
        };
        let (y, mo, d, h, mi, s, us) = timestamp.to_ymd_hms_micros();
        assert_eq!(
            CanonicalTimestamp::from_ymd_hms_micros(y, mo, d, h, mi, s, us),
            Ok(timestamp)
        );
        assert_eq!(timestamp.to_string().parse(), Ok(timestamp));
    }
    for out_of_rfc3339_range in [i64::MIN, i64::MAX] {
        let timestamp = CanonicalTimestamp {
            micros_since_unix_epoch: out_of_rfc3339_range,
        };
        assert_eq!(
            timestamp.to_rfc3339(),
            Err(CanonicalTemporalTextDenial::OutOfRange)
        );
    }
    assert_eq!(
        CanonicalTimestamp::from_ymd_hms_micros(300_000, 1, 1, 0, 0, 0, 0),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
}

#[test]
fn civil_date_parse_iso_add_days_and_start_of_day_utc_are_exact_and_checked() {
    let leap = CanonicalDate::parse_iso("2024-02-28").unwrap();
    assert_eq!(leap.add_days(1), CanonicalDate::parse_iso("2024-02-29"));
    assert_eq!(leap.add_days(2), CanonicalDate::parse_iso("2024-03-01"));
    assert_eq!(leap.add_days(-28), CanonicalDate::parse_iso("2024-01-31"));

    let non_leap_century = CanonicalDate::parse_iso("1900-02-28").unwrap();
    assert_eq!(
        non_leap_century.add_days(1),
        CanonicalDate::parse_iso("1900-03-01")
    );
    let leap_century = CanonicalDate::parse_iso("2000-02-28").unwrap();
    assert_eq!(
        leap_century.add_days(1),
        CanonicalDate::parse_iso("2000-02-29")
    );

    assert_eq!(
        CanonicalDate::parse_iso("1970-01-01").unwrap().add_days(-1),
        CanonicalDate::parse_iso("1969-12-31")
    );
    assert_eq!(
        CanonicalDate::parse_iso("not-a-date"),
        Err(CanonicalTemporalTextDenial::InvalidDate)
    );
    assert_eq!(
        CanonicalDate {
            days_from_unix_epoch: i32::MAX,
        }
        .add_days(1),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
    assert_eq!(
        CanonicalDate {
            days_from_unix_epoch: i32::MIN,
        }
        .add_days(-1),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
    assert_eq!(
        leap.add_days(i64::MAX),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );

    for iso in ["1969-12-31", "1970-01-01", "2024-02-29", "2026-11-24"] {
        let date = CanonicalDate::parse_iso(iso).unwrap();
        let midnight = date.start_of_day_utc().unwrap();
        assert_eq!(midnight.utc_date(), date);
        let (y, m, d, h, mi, s, us) = midnight.to_ymd_hms_micros();
        assert_eq!(((y, m, d), (h, mi, s, us)), (date.to_ymd(), (0, 0, 0, 0)));
        assert_eq!(midnight.to_string(), format!("{iso}T00:00:00Z"));
    }
    assert_eq!(
        CanonicalDate {
            days_from_unix_epoch: i32::MAX,
        }
        .start_of_day_utc(),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
    assert_eq!(
        CanonicalDate {
            days_from_unix_epoch: i32::MIN,
        }
        .start_of_day_utc(),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
}
