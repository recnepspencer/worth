//! Native scalar projection preserves ordinary meaning and original-request refusal.
use super::*;

#[test]
fn primitive_admitted_decoding_preserves_values_and_honors_stop_before_copy() {
    let text = AspectValue::String(InternedString::Raw("naïve\0name".into()));
    assert_eq!(
        StringApplicationValueBinding::decode_admitted(&text, &mut |_, _| Ok(())).unwrap(),
        "naïve\0name"
    );
    assert!(BoolApplicationValueBinding::decode_admitted(
        &AspectValue::Bool(true),
        &mut |_, _| Ok(())
    )
    .unwrap());
    assert_eq!(
        U64ApplicationValueBinding::decode_admitted(
            &AspectValue::UInt64(u64::MAX),
            &mut |_, _| Ok(())
        )
        .unwrap(),
        u64::MAX
    );
    assert!(matches!(
        StringApplicationValueBinding::decode_admitted(&text, &mut |_, _| Err(
            worth_foundational::facade::ValueCodecDenial::Admission
        )),
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: worth_foundational::facade::ValueCodecDenial::Admission,
            ..
        })
    ));
    assert!(matches!(
        BoolApplicationValueBinding::decode_admitted(&text, &mut |_, _| panic!(
            "a wrong family refuses before admission"
        )),
        Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch { .. })
    ));
}
