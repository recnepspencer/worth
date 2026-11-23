use super::*;

#[test]
fn civil_date_uses_the_proleptic_gregorian_calendar() {
    for (text, days) in [
        ("1969-12-31", -1),
        ("1970-01-01", 0),
        ("1900-03-01", -25_508),
        ("2000-02-29", 11_016),
        ("2026-02-28", 20_512),
    ] {
        let date = CanonicalDate {
            days_from_unix_epoch: days,
        };
        assert_eq!(text.parse(), Ok(date));
        assert_eq!(date.to_string(), text);
    }
    for text in [
        "1900-02-29",
        "2000-02-30",
        "2026-13-01",
        "20260228",
        "2026-02-28Z",
    ] {
        assert_eq!(
            text.parse::<CanonicalDate>(),
            Err(CanonicalTemporalTextDenial::InvalidDate)
        );
    }
    assert!(CanonicalDate::from_ymd(0, 2, 29).is_ok());
    assert_eq!(
        CanonicalDate::from_ymd(i32::MAX, 1, 1),
        Err(CanonicalTemporalTextDenial::OutOfRange)
    );
    for days_from_unix_epoch in [i32::MIN, i32::MAX] {
        let date = CanonicalDate {
            days_from_unix_epoch,
        };
        assert_eq!(date.to_string().parse(), Ok(date));
    }
}

#[test]
fn timestamp_text_preserves_offsets_precision_and_integer_endpoints() {
    let before_epoch = CanonicalTimestamp {
        micros_since_unix_epoch: -1,
    };
    assert_eq!(
        before_epoch.to_rfc3339().unwrap(),
        "1969-12-31T23:59:59.999999Z"
    );
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1970-01-01T00:59:59.999999+01:00"),
        Ok(before_epoch)
    );
    assert_eq!(
        CanonicalTimestamp::from_rfc3339("1969-12-31T22:59:59.9999990-01:00"),
        Ok(before_epoch)
    );
    for invalid in [
        "1970-01-01T00:00:00",
        "1970-01-01T00:00:60Z",
        "1970-01-01T00:00:00.1234567Z",
        "1970-01-01T24:00:00Z",
        "1970-01-01T00:00:00+04:60",
    ] {
        assert_eq!(
            CanonicalTimestamp::from_rfc3339(invalid),
            Err(CanonicalTemporalTextDenial::InvalidTime)
        );
    }
    for micros_since_unix_epoch in [i64::MIN, i64::MAX] {
        let timestamp = CanonicalTimestamp {
            micros_since_unix_epoch,
        };
        assert_eq!(timestamp.to_string().parse(), Ok(timestamp));
        assert_eq!(
            timestamp.to_rfc3339(),
            Err(CanonicalTemporalTextDenial::OutOfRange)
        );
    }
}
