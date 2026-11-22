//! Native carriers select losslessly by declared family and refuse
//! non-canonical or foreign compounds instead of coercing them.
use super::super::{canonical_identity, canonical_identity_with_capture, encoder};
use super::*;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use worth_foundational::facade::{
    CanonicalDate, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz,
};

fn capture(value: &impl Serialize) -> ApplicationCanonicalInput {
    let original = canonical_identity("native", "input", value).unwrap();
    let captured = canonical_identity_with_capture(
        "native",
        "input",
        value,
        encoder::HashingSink::new().capturing(true),
    )
    .unwrap();
    assert_eq!(original.identity(), captured.canonical.identity());
    captured.capture.unwrap()
}

#[derive(Serialize)]
struct Carriers {
    due: CanonicalDate,
    closed: Option<CanonicalDate>,
    opens: CanonicalTime,
    stamped: CanonicalTimestamp,
    zoned: Option<CanonicalTimestampTz>,
    id: [u8; 16],
    payload: InlineBytes,
    following: u64,
}

fn carriers(closed: Option<i32>, zoned: Option<i32>) -> Carriers {
    Carriers {
        due: CanonicalDate {
            days_from_unix_epoch: -719_468,
        },
        closed: closed.map(|days_from_unix_epoch| CanonicalDate {
            days_from_unix_epoch,
        }),
        opens: CanonicalTime::new(CanonicalTime::NANOS_PER_DAY - 1).unwrap(),
        stamped: CanonicalTimestamp {
            micros_since_unix_epoch: i64::MIN,
        },
        zoned: zoned.map(|offset_minutes| CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 1_700_000_000_000_000,
            offset_minutes,
        }),
        id: *b"0123456789abcdef",
        payload: InlineBytes::try_new(vec![0, 127, 128, 255]).unwrap(),
        following: 42,
    }
}

fn select(
    input: &ApplicationCanonicalInput,
    member: &str,
    family: ScalarAspectType,
) -> AspectValue {
    input.scalar_member(member, family).unwrap().into_value()
}

#[test]
fn native_carriers_select_required_and_optional_members_losslessly() {
    use ScalarAspectType as F;
    for (closed, zoned) in [
        (None, None),
        (Some(i32::MAX), Some(-1_439)),
        (Some(0), Some(1_439)),
    ] {
        let fixture = carriers(closed, zoned);
        let input = capture(&fixture);
        assert_eq!(
            select(&input, "due", F::Date),
            AspectValue::Date(fixture.due)
        );
        assert_eq!(
            select(&input, "opens", F::Time),
            AspectValue::Time(fixture.opens)
        );
        assert_eq!(
            select(&input, "stamped", F::Timestamp),
            AspectValue::Timestamp(fixture.stamped)
        );
        assert_eq!(select(&input, "id", F::Uuid), AspectValue::Uuid(fixture.id));
        let payload = input.scalar_member("payload", F::InlineBytes).unwrap();
        assert_eq!(payload.owned_allocation_capacity_bytes(), 4);
        assert_eq!(
            payload.into_value(),
            AspectValue::InlineBytes(fixture.payload.clone())
        );
        let optional = |member, family| {
            input
                .optional_scalar_member(member, family)
                .unwrap()
                .map(ApplicationCanonicalInputScalar::into_value)
        };
        assert_eq!(
            optional("closed", F::Date),
            fixture.closed.map(AspectValue::Date)
        );
        assert_eq!(
            optional("zoned", F::TimestampTz),
            fixture.zoned.map(AspectValue::TimestampTz)
        );
        assert_eq!(
            select(&input, "following", F::UInt64),
            AspectValue::UInt64(42)
        );
    }
}

#[test]
fn carriers_never_cross_families_or_accept_bare_numbers() {
    use ScalarAspectType as F;
    let input = capture(&carriers(Some(3), Some(60)));
    for (member, family) in [
        ("due", F::Time),
        ("opens", F::Date),
        ("following", F::Date),
        ("id", F::InlineBytes),
        ("payload", F::Uuid),
        ("due", F::Int32),
    ] {
        assert!(
            matches!(
                input.scalar_member(member, family),
                Err(ApplicationCanonicalInputDenial::InvalidScalar)
            ),
            "{member} as {family:?}"
        );
    }
}

/// Serializes the exact carrier shape with a payload no carrier can hold.
struct Forged {
    carrier: &'static str,
    fields: &'static [(&'static str, i64)],
}

impl Serialize for Forged {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut carrier = serializer.serialize_struct(self.carrier, self.fields.len())?;
        for (name, value) in self.fields {
            carrier.serialize_field(name, value)?;
        }
        carrier.end()
    }
}

#[derive(Serialize)]
struct Holder {
    value: Forged,
}

#[test]
fn non_canonical_or_foreign_carriers_refuse_with_a_typed_denial() {
    use ApplicationCanonicalInputDenial as Denial;
    use ScalarAspectType as F;
    let cases: [(&'static str, &'static [(&'static str, i64)], F, Denial); 6] = [
        (
            "CanonicalDate",
            &[("days_from_unix_epoch", 1 << 40)],
            F::Date,
            Denial::InvalidScalar,
        ),
        (
            "CanonicalTime",
            &[("nanos_since_midnight", 86_400_000_000_000)],
            F::Time,
            Denial::InvalidScalar,
        ),
        (
            "CanonicalTime",
            &[("nanos_since_midnight", -1)],
            F::Time,
            Denial::InvalidScalar,
        ),
        (
            "CanonicalTimestampTz",
            &[
                ("utc_micros_since_unix_epoch", 0),
                ("offset_minutes", 1_440),
            ],
            F::TimestampTz,
            Denial::InvalidScalar,
        ),
        (
            "CanonicalDate",
            &[("days", 1)],
            F::Date,
            Denial::UnsupportedShape,
        ),
        (
            "Deadline",
            &[("days_from_unix_epoch", 1)],
            F::Date,
            Denial::UnsupportedShape,
        ),
    ];
    for (carrier, fields, family, expected) in cases {
        let input = capture(&Holder {
            value: Forged { carrier, fields },
        });
        assert_eq!(
            input.scalar_member("value", family).err(),
            Some(expected),
            "{carrier} {fields:?}"
        );
    }
}

#[test]
fn byte_sequences_refuse_elements_wider_than_a_byte() {
    #[derive(Serialize)]
    struct Wide {
        payload: Vec<u16>,
        id: [u16; 16],
    }
    let input = capture(&Wide {
        payload: vec![1, 256],
        id: [300; 16],
    });
    for (member, family) in [
        ("payload", ScalarAspectType::InlineBytes),
        ("id", ScalarAspectType::Uuid),
    ] {
        assert!(matches!(
            input.scalar_member(member, family),
            Err(ApplicationCanonicalInputDenial::InvalidScalar)
        ));
    }
}

#[test]
fn skipping_a_member_checks_its_shape_and_only_selection_admits_its_value() {
    #[derive(Serialize)]
    struct Mixed {
        due: CanonicalDate,
        opens: CanonicalTime,
    }
    let input = capture(&Mixed {
        due: CanonicalDate {
            days_from_unix_epoch: 3,
        },
        opens: CanonicalTime {
            nanos_since_midnight: CanonicalTime::NANOS_PER_DAY,
        },
    });
    assert_eq!(
        select(&input, "due", ScalarAspectType::Date),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 3
        })
    );
    assert!(matches!(
        input.scalar_member("opens", ScalarAspectType::Time),
        Err(ApplicationCanonicalInputDenial::InvalidScalar)
    ));
}
