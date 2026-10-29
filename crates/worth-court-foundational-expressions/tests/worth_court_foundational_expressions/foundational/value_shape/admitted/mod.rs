// Representation bounds do not replace cumulative borrowed request authority.
#[allow(unused_imports)] use crate::value_shape::ordinary_context::admitted::*;
pub(crate) use {crate::value_shape::ordinary_context::packet, crate::value_shape::ordinary_context::Packet, crate::value_shape::ordinary_context::BOUNDS, crate::value_shape::ordinary_context::RAW};
pub(crate) use {worth_foundational::facade::decode_canonical_value_with_admission, worth_foundational::facade::CanonicalValue, worth_foundational::facade::ValueCodecDenial as D, worth_foundational::facade::ValueDecodeAdmission, worth_foundational::facade::ValueDecodeBounds, worth_foundational::facade::ValueShapeBody, worth_foundational::facade::WorthValue};

#[test]
fn one_loan_visits_nested_values_and_owned_allocations_without_parser_quotes() {
    let mut calls = Vec::new();
    let value = decode_canonical_value_with_admission::<Packet>(RAW, BOUNDS, &mut |work, bytes| {
        calls.push((work, bytes));
        Ok(())
    })
    .unwrap();
    assert_eq!(value, packet());
    assert_eq!(calls[0], (RAW.len(), 0));
    assert!(calls.iter().skip(1).any(|&(_, bytes)| bytes > 0));

    let escaped = r#""line\nδ""#;
    let mut refused = false;
    assert_eq!(
        decode_canonical_value_with_admission::<String>(escaped, BOUNDS, &mut |_, bytes| {
            assert!(!refused);
            if bytes > 0 {
                refused = true;
                Err(worth_foundational::facade::ValueCodecDenial::Admission)
            } else {
                Ok(())
            }
        }),
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
    assert!(refused);
    assert_eq!(
        decode_canonical_value_with_admission::<String>(escaped, BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        "line\nδ"
    );
}

#[test]
fn swallowed_seed_refusal_wins_and_new_funded_loan_succeeds() {
    let mut requests = 0;
    let mut refused = false;
    assert_eq!(
        decode_canonical_value_with_admission::<Swallowing>("true", BOUNDS, &mut |_, bytes| {
            requests += 1;
            assert!(
                !refused,
                "authority must not be re-entered after its first refusal"
            );
            if bytes == 1000 {
                refused = true;
                Err(worth_foundational::facade::ValueCodecDenial::Admission)
            } else {
                Ok(())
            }
        }),
        Err(worth_foundational::facade::ValueCodecDenial::Admission)
    );
    assert!(refused);
    assert!(requests > 1);
    assert_eq!(
        decode_canonical_value_with_admission::<Swallowing>("true", BOUNDS, &mut |_, _| Ok(()))
            .unwrap(),
        Swallowing(true)
    );
}

#[test]
fn generated_grammar_diagnostics_never_render_static_names_or_lists() {
    for raw in ["{}", r#"{"unknown":true}"#] {
        let mut decoder = serde_json::Deserializer::from_str(raw);
        let error = crate::value_shape::ordinary_context::admitted::LongGrammar::deserialize_bounded(&mut decoder, BOUNDS).unwrap_err();
        assert!(error.to_string().starts_with(&worth_foundational::facade::ValueCodecDenial::Malformed.to_string()));
    }
}

#[test]
fn opaque_owner_without_admitted_seed_has_no_deserialize_fallback() {
    assert_eq!(
        worth_foundational::facade::decode_canonical_value::<Unadmitted>("true", BOUNDS),
        Ok(Unadmitted(true))
    );
    assert_eq!(
        decode_canonical_value_with_admission::<Unadmitted>("true", BOUNDS, &mut |_, _| panic!(
            "unsupported codec must refuse before parser admission"
        )),
        Err(worth_foundational::facade::ValueCodecDenial::UnsupportedAdmission)
    );
}
