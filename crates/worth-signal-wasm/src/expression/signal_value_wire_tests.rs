use super::model::SignalValue;

fn decode(json: &str) -> SignalValue {
    serde_json::from_str(json).expect("valid JSON should decode as a signal value")
}

#[test]
fn every_json_shape_decodes_to_its_signal_value() {
    assert_eq!(decode("null"), SignalValue::Null);
    assert_eq!(decode("true"), SignalValue::Bool(true));
    assert_eq!(decode("-7"), SignalValue::Number(-7.0));
    assert_eq!(
        decode("18446744073709551615"),
        SignalValue::Number(u64::MAX as f64)
    );
    assert_eq!(decode("1.5e300"), SignalValue::Number(1.5e300));
    assert_eq!(
        decode("\"caf\\u00e9 \\ud83d\\ude00\""),
        SignalValue::String("café 😀".to_owned())
    );
    assert_eq!(
        decode(r#"[null, [false, "x"], {"k": 1}]"#),
        SignalValue::Array(vec![
            SignalValue::Null,
            SignalValue::Array(vec![
                SignalValue::Bool(false),
                SignalValue::String("x".to_owned()),
            ]),
            SignalValue::Object(vec![("k".to_owned(), SignalValue::Number(1.0))]),
        ])
    );
}

#[test]
fn object_fields_keep_canonical_key_order_and_the_last_repeated_key() {
    assert_eq!(
        decode(r#"{"b": 1, "a": {"d": [], "c": null}, "b": 2}"#),
        SignalValue::Object(vec![
            (
                "a".to_owned(),
                SignalValue::Object(vec![
                    ("c".to_owned(), SignalValue::Null),
                    ("d".to_owned(), SignalValue::Array(Vec::new())),
                ]),
            ),
            ("b".to_owned(), SignalValue::Number(2.0)),
        ])
    );
}

#[test]
fn deep_nesting_round_trips_through_json_and_the_portable_wire() {
    let mut json = String::from("0");
    for depth in 0..32 {
        json = format!(r#"{{"level": {depth}, "next": [{json}]}}"#);
    }
    let value = decode(&json);
    let reencoded = serde_json::to_string(&value).expect("signal values serialize as JSON");
    assert_eq!(decode(&reencoded), value);
    let portable = rmp_serde::to_vec_named(&value).expect("signal values serialize portably");
    let restored: SignalValue =
        rmp_serde::from_slice(&portable).expect("portable signal values decode");
    assert_eq!(restored, value);
}

#[test]
fn object_keys_must_be_strings() {
    let error = rmp_serde::from_slice::<SignalValue>(
        &rmp_serde::to_vec(&std::collections::BTreeMap::from([(1_u8, 2_u8)]))
            .expect("an integer-keyed map serializes"),
    )
    .expect_err("an integer object key is not a JSON value");
    assert!(error.to_string().contains("string"), "{error}");
}
