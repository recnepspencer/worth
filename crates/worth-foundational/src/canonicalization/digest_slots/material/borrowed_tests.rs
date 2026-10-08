use crate::facade::*;

struct TextSink(String);

impl CanonicalMaterialSink for TextSink {
    type Error = ();
    fn append(&mut self, value: &str) -> Result<(), ()> {
        self.0.push_str(value);
        Ok(())
    }
    fn admit_work(&mut self, _: usize) -> Result<(), ()> {
        Ok(())
    }
    fn accounting_overflow(&mut self) {}
}

#[test]
fn borrowed_scalar_material_preserves_frozen_historical_bytes() {
    // Literal v1 material, independent of both adapters and the encoder.
    let cases = [
        (AspectValue::Null, "value.kind#4:null;"),
        (AspectValue::Bool(true), "value.kind#4:bool;value.bool#4:true;"),
        (AspectValue::Int8(-7), "value.kind#6:signed;value.width#2:i8;value.signed#2:-7;"),
        (AspectValue::Int16(-300), "value.kind#6:signed;value.width#3:i16;value.signed#4:-300;"),
        (AspectValue::Int32(-70000), "value.kind#6:signed;value.width#3:i32;value.signed#6:-70000;"),
        (AspectValue::Int64(i64::MIN), "value.kind#6:signed;value.width#3:i64;value.signed#20:-9223372036854775808;"),
        (AspectValue::UInt8(255), "value.kind#8:unsigned;value.width#2:i8;value.unsigned#3:255;"),
        (AspectValue::UInt16(65535), "value.kind#8:unsigned;value.width#3:i16;value.unsigned#5:65535;"),
        (AspectValue::UInt32(4294967295), "value.kind#8:unsigned;value.width#3:i32;value.unsigned#10:4294967295;"),
        (AspectValue::UInt64(u64::MAX), "value.kind#8:unsigned;value.width#3:i64;value.unsigned#20:18446744073709551615;"),
        (AspectValue::Float32(CanonicalF32::from_bits(0x80000000)), "value.kind#5:float;value.width#3:f32;value.float-bits#10:2147483648;"),
        (AspectValue::Float32(CanonicalF32::from_bits(0x7fc00000)), "value.kind#5:float;value.width#3:f32;value.float-bits#10:2143289344;"),
        (AspectValue::Float64(CanonicalF64::from_bits(0x8000000000000000)), "value.kind#5:float;value.width#3:f64;value.float-bits#19:9223372036854775808;"),
        (AspectValue::Decimal(CanonicalDecimal("-12.5".into())), "value.kind#7:decimal;value.decimal.raw#5:-12.5;"),
        (AspectValue::BigInt(CanonicalBigInt("12345678901234567890".into())), "value.kind#6:bigint;value.bigint.raw#20:12345678901234567890;"),
        (AspectValue::Rational(CanonicalRational { numerator: CanonicalBigInt("-25".into()), denominator: CanonicalBigInt("2".into()) }), "value.kind#8:rational;value.rational.numerator.raw#3:-25;value.rational.denominator.raw#1:2;"),
        (AspectValue::String(InternedString::Raw("β;#:\n".into())), "value.kind#4:text;value.text.raw#6:β;#:\n;"),
        (AspectValue::String(InternedString::Symbol(Symbol(42))), "value.kind#4:text;value.text.symbol#2:42;"),
        (AspectValue::Bytes(ContentRefId(8)), "value.kind#9:bytes-ref;value.bytes-ref#1:8;"),
        (AspectValue::ContentRef(ContentRefId(9)), "value.kind#11:content-ref;value.content-ref#1:9;"),
        (AspectValue::EntityRef(EntityId::new(PartitionId(3), 17, 2)), "value.kind#10:entity-ref;value.entity.partition#1:3;value.entity.slot#2:17;value.entity.generation#1:2;"),
        (AspectValue::Uuid([0xab; 16]), "value.kind#4:uuid;value.uuid#16:abababababababababababababababab;"),
        (AspectValue::Date(CanonicalDate { days_from_unix_epoch: -2 }), "value.kind#9:date-days;value.date-days#2:-2;"),
        (AspectValue::Time(CanonicalTime::new(123).unwrap()), "value.kind#10:time-nanos;value.time-nanos#3:123;"),
        (AspectValue::Timestamp(CanonicalTimestamp { micros_since_unix_epoch: -15 }), "value.kind#16:timestamp-micros;value.timestamp-micros#3:-15;"),
        (AspectValue::TimestampTz(CanonicalTimestampTz { utc_micros_since_unix_epoch: 10, offset_minutes: -60 }), "value.kind#12:timestamp-tz;value.timestamp-tz.utc-micros#2:10;value.timestamp-tz.offset-minutes#3:-60;"),
    ];
    for (value, expected) in cases {
        let mut sink = TextSink(String::new());
        write_aspect_value_identity_material(&value, &mut sink).unwrap();
        assert_eq!(sink.0, expected);
        assert_eq!(
            prepare_aspect_value_identity_basis(&value).as_str(),
            expected
        );
        // The owned basis path must use the identical historical grammar.
        assert_eq!(
            super::value_material(&canonical_basis_value_for_aspect_value(&value)),
            expected
        );
        let mut count = CanonicalMaterialByteCount::new();
        write_aspect_value_identity_material(&value, &mut count).unwrap();
        assert_eq!(count.encoded_bytes(), expected.len());
    }
}

#[test]
fn owned_basis_only_families_keep_the_shared_historical_grammar() {
    for (value, expected) in [
        (CanonicalBasisValue::SignedInteger { width: CanonicalIntegerWidth::Bits128, value: i128::MIN }, "value.kind#6:signed;value.width#4:i128;value.signed#40:-170141183460469231731687303715884105728;"),
        (CanonicalBasisValue::UnsignedInteger { width: CanonicalIntegerWidth::Bits128, value: u128::MAX }, "value.kind#8:unsigned;value.width#4:i128;value.unsigned#39:340282366920938463463374607431768211455;"),
        (CanonicalBasisValue::BytesDigest(CanonicalDigestId::new([0x12; 32])), "value.kind#12:bytes-digest;value.bytes-digest#32:1212121212121212121212121212121212121212121212121212121212121212;"),
        (CanonicalBasisValue::NestedSequence(7), "value.kind#15:nested-sequence;value.nested-sequence#1:7;"),
    ] {
        assert_eq!(super::value_material(&value), expected);
    }
}

#[test]
fn borrowed_struct_keeps_field_order_and_literal_material() {
    let value = StructAspectValue::new([
        (FieldKey::new("z").unwrap(), AspectValue::String("β".into())),
        (FieldKey::new("a").unwrap(), AspectValue::Bool(false)),
    ])
    .unwrap();
    let expected = "value.kind#6:struct;value.struct.field-count#1:2;value.struct.field#1:a;value.kind#4:bool;value.bool#5:false;value.struct.field#1:z;value.kind#4:text;value.text.raw#2:β;";
    let mut sink = TextSink(String::new());
    write_struct_aspect_value_identity_material(&value, &mut sink).unwrap();
    assert_eq!(sink.0, expected);
    assert_eq!(
        prepare_struct_aspect_value_identity_basis(&value).as_str(),
        expected
    );
    let mut count = CanonicalMaterialByteCount::new();
    write_struct_aspect_value_identity_material(&value, &mut count).unwrap();
    assert_eq!(count.encoded_bytes(), expected.len());
}

#[derive(Debug, Eq, PartialEq)]
enum Refusal {
    Work,
    Text,
    Overflow,
}

struct RefusingSink {
    prefix: String,
    refuse_work: bool,
}

impl CanonicalMaterialSink for RefusingSink {
    type Error = Refusal;
    fn append(&mut self, value: &str) -> Result<(), Refusal> {
        if value == "payload" {
            return Err(Refusal::Text);
        }
        self.prefix.push_str(value);
        Ok(())
    }
    fn admit_work(&mut self, _: usize) -> Result<(), Refusal> {
        if self.refuse_work {
            Err(Refusal::Work)
        } else {
            Ok(())
        }
    }
    fn accounting_overflow(&mut self) -> Refusal {
        Refusal::Overflow
    }
}

#[test]
fn borrowed_writer_preserves_typed_sink_refusal_and_accepted_prefix() {
    let value = AspectValue::String("payload".into());
    let mut sink = RefusingSink {
        prefix: String::new(),
        refuse_work: false,
    };
    assert_eq!(
        write_aspect_value_identity_material(&value, &mut sink),
        Err(Refusal::Text)
    );
    assert_eq!(sink.prefix, "value.kind#4:text;value.text.raw#7:");
    let mut sink = RefusingSink {
        prefix: String::new(),
        refuse_work: true,
    };
    assert_eq!(
        write_aspect_value_identity_material(&value, &mut sink),
        Err(Refusal::Work)
    );
    assert_eq!(sink.prefix, "value.kind#");
}
