use super::binding::*;
use worth_query_decl::facade::application_schema::*;

#[test]
fn pure_owner_normalizes_writes_and_supplies_terminal_meaning() {
    let first = Ratio::new(2, 4).unwrap();
    let second = Ratio::new(1, 2).unwrap();
    let encoded = RatioBinding::encode(&first).unwrap();
    assert_eq!(encoded, RatioBinding::encode(&second).unwrap());
    let ApplicationValue::InlineBytes(bytes) = &encoded else {
        panic!("binary atom became text");
    };
    // Independent literal oracle: version1, numerator1LE, denominator2LE.
    assert_eq!(bytes.as_bytes(), &[1, 1, 0, 0, 0, 2, 0, 0, 0]);
    let decoded = RatioBinding::decode(&encoded).unwrap();
    assert_eq!(decoded, first);
    assert_eq!(
        decoded.render(),
        "1/2",
        "terminal formatting belongs to the pure owner"
    );
    assert_eq!(
        RatioBinding::UNIT.unwrap().as_str(),
        "fixture.unit.ratio.v1"
    );
    assert_eq!(
        RatioBinding::FRAME.unwrap().as_str(),
        "fixture.frames.model.v1"
    );
    let contract = RatioBinding::canonical_contract().unwrap();
    assert_eq!(contract.wire_protocol(), "worth.domain-binary.v1");
    assert_eq!(contract.bounds().maximum_wire_bytes(), Some(9));
    assert!(contract.is_domain_value());
}

#[test]
fn canonical_and_binary_frames_survive_complete_portable_contracts() {
    assert_eq!(
        SpanBinding::UNIT.unwrap().as_str(),
        "fixture.unit.length.v1"
    );
    assert_eq!(
        SpanBinding::FRAME.unwrap().as_str(),
        "fixture.frames.model.v1"
    );
    let raw = SpanBinding::encode(&Span { length: 7 }).unwrap();
    assert_eq!(
        StructuredSpanBinding::encode(&Span { length: 7 }).unwrap(),
        raw
    );
    assert_eq!(StructuredSpanBinding::FRAME, SpanBinding::FRAME);
    assert_eq!(StructuredSpanBinding::UNIT, SpanBinding::UNIT);
    assert_eq!(
        StructuredSpanBinding::canonical_contract()
            .unwrap()
            .wire_protocol(),
        "worth.canonical-json.v1"
    );
    assert_eq!(raw, ApplicationValue::String(r#"{"length":7}"#.into()));
    assert_eq!(
        SpanBinding::decode_admitted(&raw, &mut |_, _| Ok(())).unwrap(),
        Span { length: 7 }
    );
    let domain = SpanBinding::canonical_contract().unwrap();
    let structured = ApplicationCanonicalValueContract::of::<Span>(domain.bounds());
    assert_eq!(domain.wire_protocol(), "worth.domain-canonical-json.v1");
    assert!(!structured.portable_reference().matches_contract(&domain));
    assert!(!domain.portable_reference().matches_contract(&structured));
    assert!(domain.portable_reference().matches_contract(&domain));
}

#[test]
fn binding_refusals_keep_the_true_subject_and_original_budget_at_its_bound() {
    let encoded = RatioBinding::encode(&Ratio::new(1, 2).unwrap()).unwrap();
    // Input9 + certified owner128 + byte comparison9+one chunk =147.
    for budget in [146usize, 147] {
        let mut remaining = budget;
        let result = RatioBinding::decode_admitted(&encoded, &mut |work, memory| {
            assert_eq!(memory, 0);
            remaining = remaining
                .checked_sub(work)
                .ok_or(ValueCodecDenial::Admission)?;
            Ok(())
        });
        if budget == 147 {
            assert_eq!(result.unwrap().render(), "1/2");
        } else {
            assert_eq!(
                result,
                Err(ApplicationValueDecodeDenial::Canonical {
                    binding_identity: RatioBinding::IDENTITY,
                    cause: ValueCodecDenial::Admission,
                })
            );
        }
    }
    assert_eq!(
        RatioBinding::decode_admitted(&encoded, &mut |_, _| Err(ValueCodecDenial::Admission)),
        Err(ApplicationValueDecodeDenial::Canonical {
            binding_identity: RatioBinding::IDENTITY,
            cause: ValueCodecDenial::Admission
        })
    );
    for (bytes, cause) in [
        (
            vec![1, 2, 0, 0, 0, 4, 0, 0, 0],
            ValueCodecDenial::NonCanonical,
        ),
        (
            vec![2, 1, 0, 0, 0, 2, 0, 0, 0],
            ValueCodecDenial::UnsupportedVersion {
                expected: 1,
                observed: 2,
            },
        ),
        (vec![1, 1], ValueCodecDenial::Malformed),
        (vec![0; 10], ValueCodecDenial::WireBytes),
    ] {
        let raw = ApplicationValue::InlineBytes(
            worth_query_decl::facade::typed::InlineBytes::try_new(bytes).unwrap(),
        );
        assert_eq!(
            RatioBinding::decode(&raw),
            Err(ApplicationValueDecodeDenial::Canonical {
                binding_identity: RatioBinding::IDENTITY,
                cause
            })
        );
        assert_eq!(
            RatioBinding::decode_admitted(&raw, &mut |_, _| Ok(())),
            Err(ApplicationValueDecodeDenial::Canonical {
                binding_identity: RatioBinding::IDENTITY,
                cause
            })
        );
    }
}
